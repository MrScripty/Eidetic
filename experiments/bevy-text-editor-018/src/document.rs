use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    TitlePage,
    SceneHeading,
    Action,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Centered,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub kind: BlockKind,
    pub text: Rope,
}

impl Block {
    pub fn new(kind: BlockKind, text: impl AsRef<str>) -> Self {
        Self {
            kind,
            text: Rope::from_str(text.as_ref()),
        }
    }

    pub fn plain_text(&self) -> String {
        self.text.to_string()
    }

    pub fn append_wrapped_line(&mut self, line: &str) {
        if self.text.len_chars() == 0 {
            self.text.insert(0, line);
            return;
        }

        let mut current = self.text.to_string();
        if current.ends_with('-') {
            current.push_str(line);
        } else {
            current.push(' ');
            current.push_str(line);
        }
        self.text = Rope::from_str(&current);
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub source_name: String,
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new(source_name: impl Into<String>, blocks: Vec<Block>) -> Self {
        Self {
            source_name: source_name.into(),
            blocks,
        }
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_wrapped_line_collapses_pdf_line_breaks() {
        let mut block = Block::new(BlockKind::Action, "The hallway creaks");

        block.append_wrapped_line("under Gomez's footsteps.");

        assert_eq!(
            block.plain_text(),
            "The hallway creaks under Gomez's footsteps."
        );
    }

    #[test]
    fn test_append_wrapped_line_preserves_hyphenated_breaks() {
        let mut block = Block::new(BlockKind::Dialogue, "twenty-");

        block.append_wrapped_line("five years");

        assert_eq!(block.plain_text(), "twenty-five years");
    }
}
