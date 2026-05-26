mod affect;
mod agent_workflow;
mod ai_context;
mod bible_graph;
mod bible_graph_defaults;
mod bible_render_graph;
mod bible_render_graph_filter;
mod change_review;
mod context_influence;
mod graph_proposal;
mod propagation_proposal;
mod script_document;
mod selected_node_editor;
mod semantic_dependency;
mod semantic_proposal;
mod story_arc;
mod timeline_command;
mod timeline_render;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use affect::{
    AcceptAffectProposalCommand, AffectConfidence, AffectContractError, AffectDependency,
    AffectDependencyEndpoint, AffectDependencyId, AffectProjection, AffectProposal,
    AffectProposalId, AffectProposalListProjection, AffectProposalSource, AffectProvenance,
    AffectTarget, AffectTraitKind, AffectValue, AffectValueId, Arousal,
    CreateAffectProposalCommand, DeleteAffectValueCommand, EmotionalIntensity, MoodLabel,
    RecordAffectDependencyCommand, RejectAffectProposalCommand, SetAffectValueCommand, Valence,
};
pub use agent_workflow::{
    AgentProposalMode, AgentRun, AgentRunId, AgentRunStatus, AgentToolArguments, AgentToolBudget,
    AgentToolCall, AgentToolCallId, AgentToolCallStatus, AgentToolDefinition, AgentToolKind,
    AgentToolManifest, AgentToolName, AgentToolRequest, AgentToolResult, AgentToolResultPayload,
    AgentToolResultStatus, AgentWorkflowContractError, AgentWorkflowDefinition, AgentWorkflowId,
    AgentWorkflowIntent, AgentWorkflowPolicy,
};
pub use ai_context::{
    AiBibleContextEdge, AiBibleContextField, AiBibleContextNode, AiBibleContextProjection,
    AiBibleContextSnapshot,
};
pub use bible_graph::{
    BibleGraphContractError, BibleGraphEdge, BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphField,
    BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNode, BibleGraphNodeCategory,
    BibleGraphNodeId, BibleGraphNodeListProjection, BibleGraphPart, BibleGraphPartId,
    BibleGraphPartKey, BibleGraphPartProjection, BibleGraphSchemaKey, BibleGraphSnapshot,
    BibleGraphSnapshotField, BibleGraphSnapshotFieldId, BibleGraphSnapshotId,
    BibleGraphSnapshotProjection, BibleNodeDetailProjection, CanonicalBibleRoot,
    CreateBibleGraphNodeCommand, DeleteBibleGraphEdgeCommand, DeleteBibleGraphNodeCommand,
    EnsureCanonicalBibleRootsCommand, SetBibleGraphEdgeCommand, SetBibleGraphFieldCommand,
    SetBibleGraphSnapshotFieldCommand, canonical_bible_root_nodes,
};
pub use bible_graph_defaults::{
    BUILTIN_BIBLE_GRAPH_SCHEMAS, BibleGraphCategoryProjection, BibleGraphFieldDefault,
    BibleGraphFieldSchemaProjection, BibleGraphPartDefault, BibleGraphPartSchemaProjection,
    BibleGraphSchemaDefault, BibleGraphSchemaListProjection, BibleGraphSchemaProjection,
    builtin_bible_graph_schema, builtin_bible_graph_schema_list_projection,
    default_part_projections_for_node,
};
pub use bible_render_graph::{
    BibleRenderGraphEdge, BibleRenderGraphInfluence, BibleRenderGraphNeighborhood,
    BibleRenderGraphNode, BibleRenderGraphPosition, BibleRenderGraphProjection,
};
pub use bible_render_graph_filter::BibleRenderGraphProjectionRequest;
pub use change_review::{ChangeReviewChange, ChangeReviewProjection};
pub use context_influence::{
    ContextEvaluation, ContextEvaluationId, ContextEvaluationTaskKind, ContextInfluenceId,
    ContextInfluenceKind, ContextInfluenceProjection, ContextInfluenceProjectionRequest,
    ContextInfluenceProvenance, ContextInfluenceRecord, ContextLayerRole, ContextStackLayer,
    ContextStackProjection, ContextStackProjectionRequest, RecordContextEvaluationCommand,
};
pub use graph_proposal::{
    CreateGraphProposalCommand, GraphProposal, GraphProposalAction, GraphProposalContractError,
    GraphProposalId, GraphProposalListProjection, GraphProposalTarget,
};
pub use propagation_proposal::{
    AcceptPropagationProposalCommand, CreatePropagationProposalCommand, PropagationProposal,
    PropagationProposalAction, PropagationProposalContractError, PropagationProposalId,
    PropagationProposalListProjection, PropagationProposalTarget, RejectPropagationProposalCommand,
    UpdatePropagationProposalCommand,
};
pub use script_document::{
    ScriptBlock, ScriptBlockId, ScriptBlockKind, ScriptBlockProjection, ScriptContractError,
    ScriptDocument, ScriptDocumentId, ScriptDocumentProjection, ScriptLock, ScriptLockId,
    ScriptPatch, ScriptPatchId, ScriptSegment, ScriptSegmentId, ScriptSegmentProjection,
    ScriptSegmentStatus, ScriptSpan, ScriptSpanId, ScriptSpanProvenance, SetScriptBlockCommand,
    SetScriptLockCommand,
};
pub use selected_node_editor::{
    SelectedNodeEditorAdjacentParents, SelectedNodeEditorNode, SelectedNodeEditorProjection,
    SelectedNodeEditorSummary,
};
pub use semantic_dependency::{
    RecordSemanticDependencyCommand, SemanticDependency, SemanticDependencyContractError,
    SemanticDependencyEndpoint, SemanticDependencyId, SemanticDependencyKind,
    SemanticDependencyProjection,
};
pub use semantic_proposal::{
    AcceptBibleReferenceProposalCommand, BibleReferenceKind, BibleReferenceProposal,
    BibleReferenceProposalListProjection, CreateBibleReferenceProposalCommand,
    RejectBibleReferenceProposalCommand, SemanticProposalContractError, SemanticProposalId,
    SemanticProposalStatus,
};
pub use story_arc::{
    CreateStoryArcCommand, DeleteStoryArcCommand, SetStoryArcMetadataCommand,
    StoryArcListProjection, StoryArcProgressionProjection,
};
pub use timeline_command::{
    ApplyTimelineChildCommand, ApplyTimelineChildrenCommand, CreateTimelineNodeCommand,
    CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand,
    SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
};
pub use timeline_render::{
    TimelineRenderAffectSample, TimelineRenderClip, TimelineRenderGap, TimelineRenderProjection,
    TimelineRenderRelationship, TimelineRenderStructureSegment, TimelineRenderTrack,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId(pub Uuid);

impl CommandId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeEventId(pub Uuid);

impl ChangeEventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectRevisionId(pub Uuid);

impl ObjectRevisionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectionVersion(pub u64);

impl ProjectionVersion {
    pub const INITIAL: Self = Self(1);

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Project,
    TimelineNode,
    TimelineTrack,
    TimelineRelationship,
    StoryArc,
    ChildPlan,
    BibleNode,
    BiblePart,
    BiblePartField,
    BibleEdge,
    BibleSnapshot,
    ScriptDocument,
    ScriptSegment,
    ScriptBlock,
    ScriptSpan,
    ScriptLock,
    SemanticProposal,
    SemanticClaim,
    SemanticDependency,
    ReferenceAsset,
    Projection,
    ContextEvaluation,
    ContextInfluence,
    GraphProposal,
    AffectDependency,
    AgentRun,
    AgentToolCall,
    AgentToolResult,
    AffectValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeEventKind {
    UserEdit,
    AiProposalCreated,
    AiProposalUpdated,
    AiProposalAccepted,
    AiProposalRejected,
    Propagation,
    AgentWorkflow,
    Undo,
    Redo,
    Import,
    SystemRepair,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum FieldValue {
    Text(String),
    Integer(i64),
    Number(f64),
    Bool(bool),
    ObjectRef { kind: ObjectKind, id: String },
    AssetRef(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDelta {
    pub field_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_value: Option<FieldValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_value: Option<FieldValue>,
    #[serde(default)]
    pub sort_order: u32,
}

impl FieldDelta {
    pub fn new(
        field_key: impl Into<String>,
        old_value: Option<FieldValue>,
        new_value: Option<FieldValue>,
    ) -> Self {
        Self {
            field_key: field_key.into(),
            old_value,
            new_value,
            sort_order: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetObjectFieldCommand {
    pub object_kind: ObjectKind,
    pub object_id: String,
    pub field_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<FieldValue>,
}

impl SetObjectFieldCommand {
    pub fn new(
        object_kind: ObjectKind,
        object_id: impl Into<String>,
        field_key: impl Into<String>,
        value: Option<FieldValue>,
    ) -> Self {
        Self {
            object_kind,
            object_id: object_id.into(),
            field_key: field_key.into(),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub id: ChangeEventId,
    pub command_id: CommandId,
    pub kind: ChangeEventKind,
    pub summary: String,
    pub created_at_ms: u64,
}

impl ChangeEvent {
    pub fn new(command_id: CommandId, kind: ChangeEventKind, summary: impl Into<String>) -> Self {
        Self {
            id: ChangeEventId::new(),
            command_id,
            kind,
            summary: summary.into(),
            created_at_ms: 0,
        }
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectRevision {
    pub id: ObjectRevisionId,
    pub object_kind: ObjectKind,
    pub object_id: String,
    pub change_event_id: ChangeEventId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_revision_id: Option<ObjectRevisionId>,
    pub operation: RevisionOperation,
    pub fields: Vec<FieldDelta>,
}

impl ObjectRevision {
    pub fn new(
        object_kind: ObjectKind,
        object_id: impl Into<String>,
        change_event_id: ChangeEventId,
        operation: RevisionOperation,
    ) -> Self {
        Self {
            id: ObjectRevisionId::new(),
            object_kind,
            object_id: object_id.into(),
            change_event_id,
            base_revision_id: None,
            operation,
            fields: Vec::new(),
        }
    }

    pub fn with_base_revision(mut self, base_revision_id: ObjectRevisionId) -> Self {
        self.base_revision_id = Some(base_revision_id);
        self
    }

    pub fn with_field(mut self, field: FieldDelta) -> Self {
        self.fields.push(field);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope<T> {
    pub id: CommandId,
    pub payload: T,
}

impl<T> CommandEnvelope<T> {
    pub fn new(payload: T) -> Self {
        Self {
            id: CommandId::new(),
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionEnvelope<T> {
    pub version: ProjectionVersion,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_event_id: Option<ChangeEventId>,
    pub payload: T,
}

impl<T> ProjectionEnvelope<T> {
    pub fn initial(payload: T) -> Self {
        Self {
            version: ProjectionVersion::INITIAL,
            change_event_id: None,
            payload,
        }
    }

    pub fn from_event(
        version: ProjectionVersion,
        change_event_id: ChangeEventId,
        payload: T,
    ) -> Self {
        Self {
            version,
            change_event_id: Some(change_event_id),
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestCommand {
        label: String,
    }

    #[test]
    fn command_envelope_round_trips() {
        let envelope = CommandEnvelope::new(TestCommand {
            label: "create node".to_string(),
        });

        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: CommandEnvelope<TestCommand> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, envelope);
    }

    #[test]
    fn object_revision_preserves_field_delta_types() {
        let event = ChangeEvent::new(CommandId::new(), ChangeEventKind::UserEdit, "edit field");
        let revision = ObjectRevision::new(
            ObjectKind::BiblePartField,
            "field-1",
            event.id,
            RevisionOperation::Update,
        )
        .with_field(FieldDelta::new(
            "weather",
            Some(FieldValue::Text("sunny".to_string())),
            Some(FieldValue::Text("rainy".to_string())),
        ))
        .with_field(FieldDelta::new(
            "is_locked",
            Some(FieldValue::Bool(false)),
            Some(FieldValue::Bool(true)),
        ));

        let json = serde_json::to_string(&revision).unwrap();
        let decoded: ObjectRevision = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, revision);
        assert_eq!(decoded.fields.len(), 2);
    }

    #[test]
    fn projection_version_advances_monotonically() {
        let initial = ProjectionEnvelope::initial("timeline");
        let next = initial.version.next();

        assert_eq!(initial.version, ProjectionVersion::INITIAL);
        assert_eq!(next, ProjectionVersion(2));
    }

    #[test]
    fn projection_envelope_can_reference_change_event() {
        let event_id = ChangeEventId::new();
        let envelope = ProjectionEnvelope::from_event(ProjectionVersion(7), event_id, "updated");

        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: ProjectionEnvelope<String> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.version, ProjectionVersion(7));
        assert_eq!(decoded.change_event_id, Some(event_id));
        assert_eq!(decoded.payload, "updated");
    }

    #[test]
    fn set_object_field_command_round_trips() {
        let command = SetObjectFieldCommand::new(
            ObjectKind::BiblePartField,
            "field-weather",
            "weather",
            Some(FieldValue::Text("rainy".to_string())),
        );

        let json = serde_json::to_string(&command).unwrap();
        let decoded: SetObjectFieldCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, command);
    }
}
