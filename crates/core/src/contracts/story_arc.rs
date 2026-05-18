use serde::{Deserialize, Serialize};

use crate::story::arc::{ArcId, ArcType, Color, StoryArc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryArcListProjection {
    pub arcs: Vec<StoryArc>,
}

impl StoryArcListProjection {
    pub fn from_arcs(arcs: &[StoryArc]) -> Self {
        Self {
            arcs: arcs.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateStoryArcCommand {
    pub arc_id: ArcId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_arc_id: Option<ArcId>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub arc_type: ArcType,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetStoryArcMetadataCommand {
    pub arc_id: ArcId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc_type: Option<ArcType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteStoryArcCommand {
    pub arc_id: ArcId,
}
