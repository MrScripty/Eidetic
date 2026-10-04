use std::fmt::Write;

use eidetic_core::{Error, ai::backend::GenerateStream};
use futures::{
    StreamExt,
    stream::{self, BoxStream},
};

// Token events are ordinarily small; allow substantial provider metadata without
// letting unterminated lines or multi-line events retain an unbounded body.
pub(super) const MAX_LINE_BYTES: usize = 256 * 1024;
pub(super) const MAX_EVENT_BYTES: usize = 1024 * 1024;
const MAX_ERROR_BYTES: usize = 1024;

struct Reader {
    body: BoxStream<'static, Result<Vec<u8>, reqwest::Error>>,
    chunk: Vec<u8>,
    scan: usize,
    line: Vec<u8>,
    data: Vec<u8>,
    completed: bool,
    provider: &'static str,
}

impl Reader {
    fn error(&self, message: impl std::fmt::Display) -> Error {
        let mut detail = ErrorDetail(String::new());
        let truncated = write!(&mut detail, "{message}").is_err();
        Error::AiBackend(format!(
            "{} stream failed: {}{}",
            self.provider,
            detail.0,
            if truncated { " [truncated]" } else { "" }
        ))
    }

    fn event(&mut self) -> Result<Option<String>, Error> {
        if self.data.is_empty() {
            return Ok(None);
        }
        let data = std::mem::take(&mut self.data);
        let text = std::str::from_utf8(&data).map_err(|error| self.error(error))?;
        if self.completed {
            return Err(self.error("received data after completion"));
        }
        if text.trim() == "[DONE]" {
            self.completed = true;
            return Ok(None);
        }
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|error| self.error(error))?;
        if let Some(error) = value.get("error") {
            return Err(self.error(error));
        }
        if let Some(text) = value
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("delta"))
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
        {
            if !text.is_empty() {
                return Ok(Some(text.to_owned()));
            }
        }
        Ok(None)
    }

    fn lines(&mut self) -> Result<Option<String>, Error> {
        // Each chunk byte is scanned once. Yield one token at a time rather than
        // collecting all events from a transport chunk in another unbounded queue.
        while self.scan < self.chunk.len() {
            let newline = self.chunk[self.scan..]
                .iter()
                .position(|byte| *byte == b'\n');
            let end = newline.map_or(self.chunk.len(), |offset| self.scan + offset);
            if end - self.scan > MAX_LINE_BYTES - self.line.len() {
                return Err(self.error("SSE line exceeds size limit"));
            }
            self.line.extend_from_slice(&self.chunk[self.scan..end]);
            self.scan = end;
            if newline.is_none() {
                break;
            }
            self.scan += 1;
            let mut line = std::mem::take(&mut self.line);
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            if line.is_empty() {
                if let Some(token) = self.event()? {
                    return Ok(Some(token));
                }
            } else if let Some(data) = line.strip_prefix(b"data:") {
                let data = data.strip_prefix(b" ").unwrap_or(data);
                let separator = usize::from(!self.data.is_empty());
                if data.len() + separator > MAX_EVENT_BYTES - self.data.len() {
                    return Err(self.error("SSE event exceeds size limit"));
                }
                if separator != 0 {
                    self.data.push(b'\n');
                }
                self.data.extend_from_slice(data);
            }
        }
        Ok(None)
    }
}

struct ErrorDetail(String);

impl std::fmt::Write for ErrorDetail {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        let remaining = MAX_ERROR_BYTES - self.0.len();
        let mut end = value.len().min(remaining);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        self.0.push_str(&value[..end]);
        if end < value.len() {
            Err(std::fmt::Error)
        } else {
            Ok(())
        }
    }
}

pub(super) fn tokens(response: reqwest::Response, provider: &'static str) -> GenerateStream {
    let reader = Reader {
        body: response
            .bytes_stream()
            .map(|chunk| chunk.map(|bytes| bytes.to_vec()))
            .boxed(),
        chunk: Vec::new(),
        scan: 0,
        line: Vec::new(),
        data: Vec::new(),
        completed: false,
        provider,
    };
    Box::pin(stream::try_unfold(reader, |mut reader| async move {
        loop {
            if let Some(token) = reader.lines()? {
                return Ok(Some((token, reader)));
            }
            match reader.body.next().await {
                Some(Ok(bytes)) => {
                    reader.chunk = bytes;
                    reader.scan = 0;
                }
                Some(Err(error)) => return Err(reader.error(error)),
                None => {
                    // Drain through HTTP EOF even after [DONE], so an incomplete
                    // declared body cannot masquerade as a successful response.
                    if !reader.completed || !reader.data.is_empty() || !reader.line.is_empty() {
                        return Err(reader.error("response ended before complete SSE termination"));
                    }
                    return Ok(None);
                }
            }
        }
    }))
}

#[cfg(test)]
#[path = "sse_tests.rs"]
mod tests;
