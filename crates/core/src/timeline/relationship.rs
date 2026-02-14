use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::story::arc::ArcId;
use crate::story::character::CharacterId;
use super::clip::ClipId;

/// Unique identifier for a relationship between clips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipId(pub Uuid);

impl RelationshipId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A visual connection drawn between two beat clips, rendered as
/// hierarchical edge-bundled curves above the arc tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: RelationshipId,
    pub from_clip: ClipId,
    pub to_clip: ClipId,
    pub relationship_type: RelationshipType,
}

impl Relationship {
    pub fn new(
        from_clip: ClipId,
        to_clip: ClipId,
        relationship_type: RelationshipType,
    ) -> Self {
        Self {
            id: RelationshipId::new(),
            from_clip,
            to_clip,
            relationship_type,
        }
    }
}

/// The semantic type of a relationship between beat clips.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// "this causes that" — beat → beat causal link.
    Causal,
    /// "these arcs intersect at this point."
    Convergence { arc_ids: Vec<ArcId> },
    /// "this character drives this beat."
    CharacterDrives { character_id: CharacterId },
    /// User-defined thematic or structural link.
    Thematic,
}
