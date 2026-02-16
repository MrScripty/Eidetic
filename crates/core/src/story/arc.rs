use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a story arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArcId(pub Uuid);

impl ArcId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A named story arc (A-plot, B-plot, etc.) that threads through the narrative.
///
/// Arcs are hierarchical tags — any story node at any level can be tagged with
/// one or more arcs. Arcs can contain sub-arcs (e.g., a character's personal arc
/// within the main A-plot).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryArc {
    pub id: ArcId,
    /// For sub-arcs: the parent arc this belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_arc_id: Option<ArcId>,
    pub name: String,
    pub description: String,
    pub arc_type: ArcType,
    pub color: Color,
}

impl StoryArc {
    pub fn new(name: impl Into<String>, arc_type: ArcType, color: Color) -> Self {
        Self {
            id: ArcId::new(),
            parent_arc_id: None,
            name: name.into(),
            description: String::new(),
            arc_type,
            color,
        }
    }

    pub fn new_sub_arc(
        name: impl Into<String>,
        arc_type: ArcType,
        color: Color,
        parent_arc_id: ArcId,
    ) -> Self {
        Self {
            id: ArcId::new(),
            parent_arc_id: Some(parent_arc_id),
            name: name.into(),
            description: String::new(),
            arc_type,
            color,
        }
    }
}

/// The role this arc plays in the story structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcType {
    APlot,
    BPlot,
    CRunner,
    Custom(String),
}

/// An RGB color used for tracks, nodes, and relationship curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Format as CSS hex string.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    // Palette defaults for the three standard arcs.
    pub const A_PLOT: Self = Self::new(100, 149, 237);  // cornflower blue
    pub const B_PLOT: Self = Self::new(119, 221, 119);  // pastel green
    pub const C_RUNNER: Self = Self::new(255, 179, 71); // pastel orange
}
