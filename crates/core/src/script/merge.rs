use serde::{Deserialize, Serialize};

/// A piece of user-written text that must appear verbatim in generated output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub text: String,
    pub position: AnchorPosition,
}

/// Where an anchor should be placed in the generated output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnchorPosition {
    /// Insert at the very beginning.
    Beginning,
    /// Insert at the very end.
    End,
    /// Insert at approximately this fraction (0.0–1.0) through the text.
    Approximate(f64),
}

/// Merge user-written anchors into AI-generated script text.
///
/// Anchors are inserted at their specified positions. If positions overlap,
/// they are inserted in order of appearance.
pub fn merge_with_anchors(generated: &str, anchors: &[Anchor]) -> String {
    if anchors.is_empty() {
        return generated.to_owned();
    }

    // Collect insertion points sorted by position.
    let mut insertions: Vec<(usize, &str)> = anchors
        .iter()
        .map(|a| {
            let byte_pos = match a.position {
                AnchorPosition::Beginning => 0,
                AnchorPosition::End => generated.len(),
                AnchorPosition::Approximate(frac) => {
                    let frac = frac.clamp(0.0, 1.0);
                    let target = (generated.len() as f64 * frac) as usize;
                    // Snap to a newline boundary to avoid splitting lines.
                    find_nearest_line_break(generated, target)
                }
            };
            (byte_pos, a.text.as_str())
        })
        .collect();

    insertions.sort_by_key(|(pos, _)| *pos);

    let mut result = String::with_capacity(generated.len() + anchors.iter().map(|a| a.text.len() + 2).sum::<usize>());
    let mut last = 0;

    for (pos, text) in &insertions {
        let pos = (*pos).min(generated.len());
        if pos > last {
            result.push_str(&generated[last..pos]);
        }
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(text);
        if !text.ends_with('\n') {
            result.push('\n');
        }
        last = pos;
    }

    if last < generated.len() {
        result.push_str(&generated[last..]);
    }

    result
}

/// Find the nearest line break boundary to `target` byte position.
fn find_nearest_line_break(text: &str, target: usize) -> usize {
    if target >= text.len() {
        return text.len();
    }

    // Search forward for the next newline.
    if let Some(fwd) = text[target..].find('\n') {
        return target + fwd + 1;
    }

    // No newline found forward; search backward.
    if let Some(bwd) = text[..target].rfind('\n') {
        return bwd + 1;
    }

    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_anchors_returns_original() {
        let text = "Hello, world.";
        assert_eq!(merge_with_anchors(text, &[]), "Hello, world.");
    }

    #[test]
    fn anchor_at_beginning() {
        let generated = "Line two.\nLine three.\n";
        let anchors = vec![Anchor {
            text: "Line one.".to_owned(),
            position: AnchorPosition::Beginning,
        }];
        let result = merge_with_anchors(generated, &anchors);
        assert!(result.starts_with("Line one."));
        assert!(result.contains("Line two."));
    }

    #[test]
    fn anchor_at_end() {
        let generated = "Line one.\nLine two.\n";
        let anchors = vec![Anchor {
            text: "Line three.".to_owned(),
            position: AnchorPosition::End,
        }];
        let result = merge_with_anchors(generated, &anchors);
        assert!(result.contains("Line one."));
        assert!(result.ends_with("Line three.\n"));
    }

    #[test]
    fn anchor_at_approximate_midpoint() {
        let generated = "Line one.\nLine two.\nLine three.\nLine four.\n";
        let anchors = vec![Anchor {
            text: "INSERTED".to_owned(),
            position: AnchorPosition::Approximate(0.5),
        }];
        let result = merge_with_anchors(generated, &anchors);
        assert!(result.contains("INSERTED"));
        // Should appear roughly in the middle.
        let pos = result.find("INSERTED").unwrap();
        assert!(pos > 5 && pos < result.len() - 5);
    }
}
