use eidetic_core::ai::backend::ChildPlanId;
use eidetic_core::contracts::{
    ApplyTimelineChildCommand, ApplyTimelineChildrenCommand, CommandEnvelope, CommandId,
    CreateTimelineNodeCommand, CreateTimelineRelationshipCommand,
};
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};
use serde::Deserialize;

use crate::backend_error::BackendError;
use crate::command_service_support::derived_command_uuid;
use crate::validation;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTimelineNodeRequestCommand {
    id: CommandId,
    payload: CreateTimelineNodeRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTimelineNodeRequestPayload {
    #[serde(default)]
    node_id: Option<NodeId>,
    parent_id: Option<NodeId>,
    level: StoryLevel,
    name: String,
    start_ms: u64,
    end_ms: u64,
    beat_type: Option<BeatType>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitTimelineNodeRequestCommand {
    id: CommandId,
    payload: SplitTimelineNodeRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTimelineRelationshipRequestCommand {
    id: CommandId,
    payload: CreateTimelineRelationshipRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyTimelineChildrenRequestCommand {
    id: CommandId,
    payload: ApplyTimelineChildrenRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyTimelineChildrenRequestPayload {
    parent_id: NodeId,
    #[serde(default)]
    child_plan_id: Option<ChildPlanId>,
    children: Vec<ApplyTimelineChildRequestPayload>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyTimelineChildRequestPayload {
    #[serde(default)]
    node_id: Option<NodeId>,
    name: String,
    outline: String,
    weight: f32,
    beat_type: Option<BeatType>,
    #[serde(default)]
    characters: Vec<String>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    props: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTimelineRelationshipRequestPayload {
    #[serde(default)]
    relationship_id: Option<RelationshipId>,
    from_node_id: NodeId,
    to_node_id: NodeId,
    relationship_type: RelationshipType,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SplitTimelineNodeRequestPayload {
    node_id: NodeId,
    at_ms: u64,
    #[serde(default)]
    left_node_id: Option<NodeId>,
    #[serde(default)]
    right_node_id: Option<NodeId>,
}

impl CreateTimelineNodeRequestCommand {
    pub(crate) fn validate(&self) -> Result<(), BackendError> {
        validation::validate_name(&self.payload.name, "node name")
    }

    pub(crate) fn into_core_command(self) -> CommandEnvelope<CreateTimelineNodeCommand> {
        CommandEnvelope {
            id: self.id,
            payload: CreateTimelineNodeCommand {
                node_id: self
                    .payload
                    .node_id
                    .unwrap_or_else(|| NodeId(derived_command_uuid(self.id, b"timeline.node"))),
                parent_id: self.payload.parent_id,
                level: self.payload.level,
                name: self.payload.name,
                start_ms: self.payload.start_ms,
                end_ms: self.payload.end_ms,
                beat_type: self.payload.beat_type,
            },
        }
    }
}

impl SplitTimelineNodeRequestCommand {
    pub(crate) fn into_core_command(
        self,
    ) -> CommandEnvelope<eidetic_core::contracts::SplitTimelineNodeCommand> {
        CommandEnvelope {
            id: self.id,
            payload: eidetic_core::contracts::SplitTimelineNodeCommand {
                node_id: self.payload.node_id,
                at_ms: self.payload.at_ms,
                left_node_id: self.payload.left_node_id.unwrap_or_else(|| {
                    NodeId(derived_command_uuid(self.id, b"timeline.split.left"))
                }),
                right_node_id: self.payload.right_node_id.unwrap_or_else(|| {
                    NodeId(derived_command_uuid(self.id, b"timeline.split.right"))
                }),
            },
        }
    }
}

impl CreateTimelineRelationshipRequestCommand {
    pub(crate) fn into_core_command(self) -> CommandEnvelope<CreateTimelineRelationshipCommand> {
        CommandEnvelope {
            id: self.id,
            payload: CreateTimelineRelationshipCommand {
                relationship_id: self.payload.relationship_id.unwrap_or_else(|| {
                    RelationshipId(derived_command_uuid(self.id, b"timeline.relationship"))
                }),
                from_node_id: self.payload.from_node_id,
                to_node_id: self.payload.to_node_id,
                relationship_type: self.payload.relationship_type,
            },
        }
    }
}

impl ApplyTimelineChildrenRequestCommand {
    pub(crate) fn validate(&self) -> Result<(), BackendError> {
        for child in &self.payload.children {
            validation::validate_name(&child.name, "child node name")?;
            validation::validate_positive_finite_f32(child.weight, "child weight")?;
        }
        Ok(())
    }

    pub(crate) fn into_core_command(self) -> CommandEnvelope<ApplyTimelineChildrenCommand> {
        CommandEnvelope {
            id: self.id,
            payload: ApplyTimelineChildrenCommand {
                parent_id: self.payload.parent_id,
                child_plan_id: self.payload.child_plan_id,
                children: self
                    .payload
                    .children
                    .into_iter()
                    .enumerate()
                    .map(|(index, child)| ApplyTimelineChildCommand {
                        node_id: child.node_id.unwrap_or_else(|| {
                            NodeId(derived_indexed_command_uuid(
                                self.id,
                                b"timeline.child",
                                index,
                            ))
                        }),
                        name: child.name,
                        outline: child.outline,
                        weight: child.weight,
                        beat_type: child.beat_type,
                        characters: child.characters,
                        location: child.location,
                        props: child.props,
                    })
                    .collect(),
            },
        }
    }
}

fn derived_indexed_command_uuid(command_id: CommandId, role: &[u8], index: usize) -> uuid::Uuid {
    let mut bytes = *derived_command_uuid(command_id, role).as_bytes();
    for (offset, byte) in index.to_le_bytes().iter().enumerate() {
        let slot = bytes.len() - 1 - (offset % bytes.len());
        bytes[slot] ^= *byte;
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes)
}
