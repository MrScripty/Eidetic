use std::path::Path;

use crate::document::{Block, BlockKind, Document};

pub fn parse_fountain(source_name: impl Into<String>, source: &str) -> Document {
    let mut parser = FountainParser::new(source_name.into());
    parser.parse(source);
    parser.finish()
}

struct FountainParser {
    source_name: String,
    blocks: Vec<Block>,
    previous_nonblank: Option<BlockKind>,
    previous_line_blank: bool,
}

impl FountainParser {
    fn new(source_name: String) -> Self {
        Self {
            source_name,
            blocks: Vec::new(),
            previous_nonblank: None,
            previous_line_blank: true,
        }
    }

    fn parse(&mut self, source: &str) {
        for raw_line in source.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                self.previous_line_blank = true;
                self.previous_nonblank = None;
                continue;
            }

            let kind = self.classify(line);
            self.push_line(kind, normalize_line(line, kind));
            self.previous_nonblank = Some(kind);
            self.previous_line_blank = false;
        }
    }

    fn classify(&self, line: &str) -> BlockKind {
        if is_title_page_line(line) {
            return BlockKind::TitlePage;
        }
        if is_centered(line) {
            return BlockKind::Centered;
        }
        if is_scene_heading(line) {
            return BlockKind::SceneHeading;
        }
        if is_transition(line) {
            return BlockKind::Transition;
        }
        if is_parenthetical(line) {
            return BlockKind::Parenthetical;
        }
        if is_character_cue(line, self.previous_line_blank) {
            return BlockKind::Character;
        }
        if matches!(
            self.previous_nonblank,
            Some(BlockKind::Character | BlockKind::Dialogue | BlockKind::Parenthetical)
        ) {
            return BlockKind::Dialogue;
        }
        if line.starts_with('!') {
            return BlockKind::Action;
        }
        BlockKind::Action
    }

    fn push_line(&mut self, kind: BlockKind, line: String) {
        let merge_with_previous = matches!(kind, BlockKind::Action | BlockKind::Dialogue)
            && self
                .blocks
                .last()
                .is_some_and(|previous| previous.kind == kind);

        if merge_with_previous {
            let previous = self
                .blocks
                .last_mut()
                .expect("checked last block before merge");
            previous.append_wrapped_line(&line);
        } else {
            self.blocks.push(Block::new(kind, line));
        }
    }

    fn finish(self) -> Document {
        Document::new(self.source_name, self.blocks)
    }
}

fn normalize_line(line: &str, kind: BlockKind) -> String {
    let line = match kind {
        BlockKind::Action if line.starts_with('!') => &line[1..],
        BlockKind::Transition if line.starts_with('>') => line[1..].trim(),
        BlockKind::Centered if line.starts_with('>') && line.ends_with('<') => {
            line[1..line.len() - 1].trim()
        }
        _ => line,
    };

    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_title_page_line(line: &str) -> bool {
    let Some((key, _)) = line.split_once(':') else {
        return false;
    };

    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "title" | "credit" | "author" | "authors" | "source" | "draft date" | "date"
    )
}

fn is_centered(line: &str) -> bool {
    line.starts_with('>') && line.ends_with('<')
}

fn is_scene_heading(line: &str) -> bool {
    let trimmed = line.trim_start_matches('.');
    let upper = trimmed.to_ascii_uppercase();
    matches!(
        upper.split_whitespace().next(),
        Some("INT.")
            | Some("EXT.")
            | Some("INT./EXT.")
            | Some("EXT./INT.")
            | Some("INT/EXT.")
            | Some("I/E.")
            | Some("EST.")
    )
}

fn is_transition(line: &str) -> bool {
    if line.starts_with('>') {
        return true;
    }

    let upper = line.to_ascii_uppercase();
    matches!(upper.as_str(), "FADE IN:" | "FADE OUT." | "THE END.")
        || (upper.ends_with(" TO:") && is_mostly_uppercase(&upper))
}

fn is_parenthetical(line: &str) -> bool {
    line.starts_with('(') && line.ends_with(')')
}

fn is_character_cue(line: &str, previous_line_blank: bool) -> bool {
    if line.starts_with('!') || !previous_line_blank {
        return false;
    }

    let line = line.trim_start_matches('@');
    let char_count = line.chars().count();
    char_count <= 38
        && !line.contains(':')
        && !line.ends_with('.')
        && is_mostly_uppercase(line)
        && line.chars().any(char::is_alphabetic)
}

fn is_mostly_uppercase(line: &str) -> bool {
    let mut alphabetic = 0usize;
    let mut uppercase = 0usize;

    for character in line.chars().filter(|character| character.is_alphabetic()) {
        alphabetic += 1;
        if character.is_uppercase() {
            uppercase += 1;
        }
    }

    alphabetic > 0 && uppercase == alphabetic
}

pub fn source_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace('_', " "))
        .unwrap_or_else(|| "Untitled".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fountain_classifies_screenplay_blocks() {
        let source = r#"Title: Test Script

FADE IN:

INT. HOUSE - NIGHT #1#
A candle gutters on the table.

GOMEZ
(delighted)
At last.

> CUT TO:
"#;

        let document = parse_fountain("fixture", source);
        let kinds = document
            .blocks
            .iter()
            .map(|block| block.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                BlockKind::TitlePage,
                BlockKind::Transition,
                BlockKind::SceneHeading,
                BlockKind::Action,
                BlockKind::Character,
                BlockKind::Parenthetical,
                BlockKind::Dialogue,
                BlockKind::Transition,
            ]
        );
    }

    #[test]
    fn test_parse_fountain_merges_wrapped_action_and_dialogue() {
        let source = r#"INT. ROOM - DAY
The old house groans
under the storm.

MARA
This is the first
line of dialogue.
"#;

        let document = parse_fountain("fixture", source);

        assert_eq!(
            document.blocks[1].plain_text(),
            "The old house groans under the storm."
        );
        assert_eq!(
            document.blocks[3].plain_text(),
            "This is the first line of dialogue."
        );
    }

    #[test]
    fn test_bang_forces_uppercase_action() {
        let document = parse_fountain("fixture", "!LOW TRACKING SHOT");

        assert_eq!(document.blocks[0].kind, BlockKind::Action);
        assert_eq!(document.blocks[0].plain_text(), "LOW TRACKING SHOT");
    }
}
