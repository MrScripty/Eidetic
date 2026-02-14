use serde::{Deserialize, Serialize};

use crate::reference::ReferenceDocument;
use crate::story::arc::StoryArc;
use crate::story::character::Character;
use crate::timeline::Timeline;

/// A complete Eidetic project, aggregating the timeline, arcs, and characters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub timeline: Timeline,
    pub arcs: Vec<StoryArc>,
    pub characters: Vec<Character>,
    #[serde(default)]
    pub references: Vec<ReferenceDocument>,
}

impl Project {
    pub fn new(name: impl Into<String>, timeline: Timeline) -> Self {
        Self {
            name: name.into(),
            timeline,
            arcs: Vec::new(),
            characters: Vec::new(),
            references: Vec::new(),
        }
    }
}
