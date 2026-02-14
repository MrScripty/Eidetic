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

/// A named story arc (A-plot, B-plot, etc.) with its own track on the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryArc {
    pub id: ArcId,
    pub name: String,
    pub description: String,
    pub arc_type: ArcType,
    pub color: Color,
}

impl StoryArc {
    pub fn new(name: impl Into<String>, arc_type: ArcType, color: Color) -> Self {
        Self {
            id: ArcId::new(),
            name: name.into(),
            description: String::new(),
            arc_type,
            color,
        }
    }
}

/// The role this arc plays in the episode structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcType {
    APlot,
    BPlot,
    CRunner,
    Custom(String),
}

/// An RGB color used for tracks, clips, and relationship curves.
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
