use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::arc::Color;

/// Unique identifier for a character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterId(pub Uuid);

impl CharacterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A character — referenced by beat clips and relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub description: String,
    /// How this character speaks (voice notes for AI generation).
    pub voice_notes: String,
    pub color: Color,
}

impl Character {
    pub fn new(name: impl Into<String>, color: Color) -> Self {
        Self {
            id: CharacterId::new(),
            name: name.into(),
            description: String::new(),
            voice_notes: String::new(),
            color,
        }
    }
}
