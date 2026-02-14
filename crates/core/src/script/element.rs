use serde::{Deserialize, Serialize};

/// A single element of formatted screenplay text.
///
/// Script elements follow standard TV screenplay conventions:
/// scene headings in ALL CAPS, character names centered and capped,
/// dialogue indented, etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptElement {
    SceneHeading(String),
    Action(String),
    Character(String),
    Parenthetical(String),
    Dialogue(String),
    Transition(String),
}

impl ScriptElement {
    /// Render this element as plain text with standard formatting.
    pub fn to_plain_text(&self) -> String {
        match self {
            Self::SceneHeading(s) => s.to_uppercase(),
            Self::Action(s) => s.clone(),
            Self::Character(s) => format!("          {}", s.to_uppercase()),
            Self::Parenthetical(s) => format!("       ({s})"),
            Self::Dialogue(s) => format!("     {s}"),
            Self::Transition(s) => format!("{:>60}", s.to_uppercase()),
        }
    }
}
