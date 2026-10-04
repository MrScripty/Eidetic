use std::collections::VecDeque;

use eidetic_core::{Error, ai::backend::GenerateStream};
use futures::{
    StreamExt,
    stream::{self, BoxStream},
};

struct Reader {
    body: BoxStream<'static, Result<Vec<u8>, reqwest::Error>>,
    bytes: Vec<u8>,
    data: Vec<u8>,
    pending: VecDeque<String>,
    completed: bool,
    provider: &'static str,
}

impl Reader {
    fn error(&self, message: impl std::fmt::Display) -> Error {
        Error::AiBackend(format!("{} stream failed: {message}", self.provider))
    }

    fn event(&mut self) -> Result<(), Error> {
        if self.data.is_empty() {
            return Ok(());
        }
        let data = std::mem::take(&mut self.data);
        let text = std::str::from_utf8(&data).map_err(|error| self.error(error))?;
        if self.completed {
            return Err(self.error("received data after completion"));
        }
        if text.trim() == "[DONE]" {
            self.completed = true;
            return Ok(());
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
                self.pending.push_back(text.to_owned());
            }
        }
        Ok(())
    }

    fn lines(&mut self) -> Result<(), Error> {
        while let Some(end) = self.bytes.iter().position(|byte| *byte == b'\n') {
            let mut line: Vec<u8> = self.bytes.drain(..=end).collect();
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            if line.is_empty() {
                self.event()?;
            } else if let Some(data) = line.strip_prefix(b"data:") {
                let data = data.strip_prefix(b" ").unwrap_or(data);
                if !self.data.is_empty() {
                    self.data.push(b'\n');
                }
                self.data.extend_from_slice(data);
            }
        }
        Ok(())
    }
}

pub(super) fn tokens(response: reqwest::Response, provider: &'static str) -> GenerateStream {
    let reader = Reader {
        body: response
            .bytes_stream()
            .map(|chunk| chunk.map(|bytes| bytes.to_vec()))
            .boxed(),
        bytes: Vec::new(),
        data: Vec::new(),
        pending: VecDeque::new(),
        completed: false,
        provider,
    };
    Box::pin(stream::try_unfold(reader, |mut reader| async move {
        loop {
            if let Some(token) = reader.pending.pop_front() {
                return Ok(Some((token, reader)));
            }
            match reader.body.next().await {
                Some(Ok(bytes)) => {
                    reader.bytes.extend(bytes);
                    reader.lines()?;
                }
                Some(Err(error)) => return Err(reader.error(error)),
                None => {
                    // Drain through HTTP EOF even after [DONE], so an incomplete
                    // declared body cannot masquerade as a successful response.
                    if !reader.completed || !reader.data.is_empty() || !reader.bytes.is_empty() {
                        return Err(reader.error("response ended before complete SSE termination"));
                    }
                    return Ok(None);
                }
            }
        }
    }))
}
