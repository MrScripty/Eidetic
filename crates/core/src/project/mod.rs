use serde::{Deserialize, Serialize};

use crate::reference::ReferenceDocument;
use crate::story::arc::StoryArc;
use crate::story::bible::StoryBible;
use crate::timeline::Timeline;

/// A complete Eidetic project, aggregating the timeline, arcs, and story bible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    /// The project-level story premise/concept.
    #[serde(default)]
    pub premise: String,
    pub timeline: Timeline,
    pub arcs: Vec<StoryArc>,
    #[serde(default)]
    pub bible: StoryBible,
    #[serde(default)]
    pub references: Vec<ReferenceDocument>,
}

impl Project {
    pub fn new(name: impl Into<String>, timeline: Timeline) -> Self {
        Self {
            name: name.into(),
            premise: String::new(),
            timeline,
            arcs: Vec::new(),
            bible: StoryBible::new(),
            references: Vec::new(),
        }
    }
}
