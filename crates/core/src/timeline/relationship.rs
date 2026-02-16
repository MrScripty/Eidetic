use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::story::arc::ArcId;
use crate::story::bible::EntityId;
use super::node::NodeId;

/// Unique identifier for a relationship between nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipId(pub Uuid);

impl RelationshipId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A visual connection drawn between two story nodes, rendered as
/// hierarchical edge-bundled curves on the timeline.
///
/// Relationships are valid between nodes at any hierarchy level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: RelationshipId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub relationship_type: RelationshipType,
}

impl Relationship {
    pub fn new(
        from_node: NodeId,
        to_node: NodeId,
        relationship_type: RelationshipType,
    ) -> Self {
        Self {
            id: RelationshipId::new(),
            from_node,
            to_node,
            relationship_type,
        }
    }
}

/// The semantic type of a relationship between story nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// "this causes that" — causal link between nodes.
    Causal,
    /// "these arcs intersect at this point."
    Convergence { arc_ids: Vec<ArcId> },
    /// "this entity drives this node."
    EntityDrives { entity_id: EntityId },
    /// User-defined thematic or structural link.
    Thematic,
}
