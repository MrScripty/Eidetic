use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::timeline::node::NodeId;

use super::{
    BibleGraphEdgeKind, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey,
    BibleGraphSchemaKey, CommandId, ContextEvaluationTaskKind,
};

const DEFAULT_MAX_TOOL_CALLS: u32 = 32;
const DEFAULT_MAX_GRAPH_READ_LIMIT: u32 = 64;
const DEFAULT_MAX_NEIGHBORHOOD_DEPTH: u8 = 2;
const DEFAULT_MAX_RESULT_BYTES: u32 = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentRunId(pub Uuid);

impl AgentRunId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentToolCallId(pub Uuid);

impl AgentToolCallId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AgentWorkflowId(String);

impl AgentWorkflowId {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentWorkflowContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AgentWorkflowContractError::EmptyIdentifier(
                "AgentWorkflowId",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AgentWorkflowId {
    type Error = AgentWorkflowContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<AgentWorkflowId> for String {
    fn from(value: AgentWorkflowId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AgentToolName(String);

impl AgentToolName {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentWorkflowContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AgentWorkflowContractError::EmptyIdentifier("AgentToolName"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AgentToolName {
    type Error = AgentWorkflowContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<AgentToolName> for String {
    fn from(value: AgentToolName) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentWorkflowDefinition {
    pub id: AgentWorkflowId,
    pub label: String,
    pub intent: AgentWorkflowIntent,
    pub manifest: AgentToolManifest,
    pub budget: AgentToolBudget,
    pub policy: AgentWorkflowPolicy,
}

impl AgentWorkflowDefinition {
    pub fn validate(&self) -> Result<(), AgentWorkflowContractError> {
        self.budget.validate()?;
        self.manifest.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentWorkflowIntent {
    DevelopPremiseGraphContext,
    RefineActContext,
    RefineSequenceContext,
    RefineSceneContext,
    RefineBeatContext,
    RefineShotContext,
    InspectGraphContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentToolManifest {
    #[serde(default)]
    pub tools: Vec<AgentToolDefinition>,
}

impl AgentToolManifest {
    pub fn validate(&self) -> Result<(), AgentWorkflowContractError> {
        for (index, tool) in self.tools.iter().enumerate() {
            if self.tools[index + 1..]
                .iter()
                .any(|candidate| candidate.name == tool.name)
            {
                return Err(AgentWorkflowContractError::DuplicateTool {
                    tool_name: tool.name.clone(),
                });
            }
        }
        Ok(())
    }

    pub fn validate_call(
        &self,
        request: &AgentToolRequest,
        budget: &AgentToolBudget,
    ) -> Result<(), AgentWorkflowContractError> {
        let Some(tool) = self
            .tools
            .iter()
            .find(|tool| tool.name == request.tool_name)
        else {
            return Err(AgentWorkflowContractError::ToolNotAllowed {
                tool_name: request.tool_name.clone(),
            });
        };
        if tool.kind != request.arguments.kind() {
            return Err(AgentWorkflowContractError::ToolKindMismatch {
                tool_name: request.tool_name.clone(),
                expected: tool.kind,
                actual: request.arguments.kind(),
            });
        }
        request.arguments.validate(budget)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentToolDefinition {
    pub name: AgentToolName,
    pub kind: AgentToolKind,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentToolKind {
    GraphRead,
    GraphProposal,
    ContextEvaluation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentToolBudget {
    pub max_tool_calls: u32,
    pub max_graph_read_limit: u32,
    pub max_neighborhood_depth: u8,
    pub max_result_bytes: u32,
}

impl Default for AgentToolBudget {
    fn default() -> Self {
        Self {
            max_tool_calls: DEFAULT_MAX_TOOL_CALLS,
            max_graph_read_limit: DEFAULT_MAX_GRAPH_READ_LIMIT,
            max_neighborhood_depth: DEFAULT_MAX_NEIGHBORHOOD_DEPTH,
            max_result_bytes: DEFAULT_MAX_RESULT_BYTES,
        }
    }
}

impl AgentToolBudget {
    pub fn validate(&self) -> Result<(), AgentWorkflowContractError> {
        if self.max_tool_calls == 0 {
            return Err(AgentWorkflowContractError::UnboundedBudget(
                "max_tool_calls",
            ));
        }
        if self.max_graph_read_limit == 0 {
            return Err(AgentWorkflowContractError::UnboundedBudget(
                "max_graph_read_limit",
            ));
        }
        if self.max_result_bytes == 0 {
            return Err(AgentWorkflowContractError::UnboundedBudget(
                "max_result_bytes",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentWorkflowPolicy {
    pub proposal_mode: AgentProposalMode,
    #[serde(default)]
    pub allow_canonical_commits: bool,
    #[serde(default)]
    pub require_reviewable_outputs: bool,
}

impl Default for AgentWorkflowPolicy {
    fn default() -> Self {
        Self {
            proposal_mode: AgentProposalMode::ReviewOnly,
            allow_canonical_commits: false,
            require_reviewable_outputs: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentProposalMode {
    ReviewOnly,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRun {
    pub id: AgentRunId,
    pub workflow_id: AgentWorkflowId,
    pub status: AgentRunStatus,
    pub intent: AgentWorkflowIntent,
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentToolRequest {
    pub tool_name: AgentToolName,
    pub arguments: AgentToolArguments,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentToolCall {
    pub id: AgentToolCallId,
    pub run_id: AgentRunId,
    pub sequence: u32,
    pub request: AgentToolRequest,
    pub status: AgentToolCallStatus,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentToolCallStatus {
    Pending,
    Running,
    Completed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentToolArguments {
    SearchBibleNodes {
        query: String,
        limit: u32,
    },
    ReadBibleNode {
        node_id: BibleGraphNodeId,
    },
    ReadBibleNeighborhood {
        node_id: BibleGraphNodeId,
        depth: u8,
        limit: u32,
    },
    ReadContextStack {
        target_node_id: NodeId,
    },
    ReadActiveGraphContext {
        target_node_id: NodeId,
    },
    ReadInfluencePaths {
        target_node_id: NodeId,
        limit: u32,
    },
    ProposeBibleNode {
        command_id: CommandId,
        parent_id: BibleGraphNodeId,
        schema_key: BibleGraphSchemaKey,
        title: String,
        summary: String,
    },
    ProposeBibleField {
        command_id: CommandId,
        node_id: BibleGraphNodeId,
        part_key: BibleGraphPartKey,
        field_key: BibleGraphFieldKey,
        value: String,
    },
    ProposeBibleEdge {
        command_id: CommandId,
        from_node_id: BibleGraphNodeId,
        to_node_id: BibleGraphNodeId,
        edge_kind: BibleGraphEdgeKind,
        label: String,
    },
    ProposeTimelineContextLink {
        command_id: CommandId,
        timeline_node_id: NodeId,
        bible_node_id: BibleGraphNodeId,
        rationale: String,
    },
    RecordContextEvaluation {
        command_id: CommandId,
        target_node_id: NodeId,
        task_kind: ContextEvaluationTaskKind,
        summary: String,
    },
}

impl AgentToolArguments {
    pub fn kind(&self) -> AgentToolKind {
        match self {
            Self::SearchBibleNodes { .. }
            | Self::ReadBibleNode { .. }
            | Self::ReadBibleNeighborhood { .. }
            | Self::ReadContextStack { .. }
            | Self::ReadActiveGraphContext { .. }
            | Self::ReadInfluencePaths { .. } => AgentToolKind::GraphRead,
            Self::ProposeBibleNode { .. }
            | Self::ProposeBibleField { .. }
            | Self::ProposeBibleEdge { .. }
            | Self::ProposeTimelineContextLink { .. } => AgentToolKind::GraphProposal,
            Self::RecordContextEvaluation { .. } => AgentToolKind::ContextEvaluation,
        }
    }

    pub fn validate(&self, budget: &AgentToolBudget) -> Result<(), AgentWorkflowContractError> {
        match self {
            Self::SearchBibleNodes { query, limit } => {
                validate_non_empty("query", query)?;
                validate_limit("limit", *limit, budget.max_graph_read_limit)
            }
            Self::ReadBibleNeighborhood { depth, limit, .. } => {
                if *depth == 0 || *depth > budget.max_neighborhood_depth {
                    return Err(AgentWorkflowContractError::ReadOutOfBounds {
                        field: "depth",
                        requested: u32::from(*depth),
                        max: u32::from(budget.max_neighborhood_depth),
                    });
                }
                validate_limit("limit", *limit, budget.max_graph_read_limit)
            }
            Self::ReadInfluencePaths { limit, .. } => {
                validate_limit("limit", *limit, budget.max_graph_read_limit)
            }
            Self::ProposeBibleNode { title, .. } => validate_non_empty("title", title),
            Self::ProposeBibleField { value, .. } => validate_non_empty("value", value),
            Self::ProposeBibleEdge { label, .. } => validate_non_empty("label", label),
            Self::ProposeTimelineContextLink { rationale, .. } => {
                validate_non_empty("rationale", rationale)
            }
            Self::RecordContextEvaluation { summary, .. } => validate_non_empty("summary", summary),
            Self::ReadBibleNode { .. }
            | Self::ReadContextStack { .. }
            | Self::ReadActiveGraphContext { .. } => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentToolResult {
    pub call_id: AgentToolCallId,
    pub status: AgentToolResultStatus,
    pub payload: AgentToolResultPayload,
    pub completed_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentToolResultStatus {
    Succeeded,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentToolResultPayload {
    Text { text: String },
    Rejection { reason: String },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AgentWorkflowContractError {
    #[error("empty identifier for {0}")]
    EmptyIdentifier(&'static str),
    #[error("budget field {0} must be greater than zero")]
    UnboundedBudget(&'static str),
    #[error("duplicate tool {tool_name:?} in manifest")]
    DuplicateTool { tool_name: AgentToolName },
    #[error("tool {tool_name:?} is not allowed by workflow manifest")]
    ToolNotAllowed { tool_name: AgentToolName },
    #[error("tool {tool_name:?} expected kind {expected:?} but received {actual:?}")]
    ToolKindMismatch {
        tool_name: AgentToolName,
        expected: AgentToolKind,
        actual: AgentToolKind,
    },
    #[error("tool argument {field} must be greater than zero and no more than {max}")]
    ReadOutOfBounds {
        field: &'static str,
        requested: u32,
        max: u32,
    },
    #[error("tool argument {0} must not be empty")]
    EmptyArgument(&'static str),
}

fn validate_limit(
    field: &'static str,
    requested: u32,
    max: u32,
) -> Result<(), AgentWorkflowContractError> {
    if requested == 0 || requested > max {
        return Err(AgentWorkflowContractError::ReadOutOfBounds {
            field,
            requested,
            max,
        });
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), AgentWorkflowContractError> {
    if value.trim().is_empty() {
        return Err(AgentWorkflowContractError::EmptyArgument(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_contract_round_trips_with_manifest_policy_and_budget() {
        let workflow = AgentWorkflowDefinition {
            id: AgentWorkflowId::new("workflow.premise.graph").unwrap(),
            label: "Premise graph development".to_string(),
            intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
            manifest: AgentToolManifest {
                tools: vec![AgentToolDefinition {
                    name: AgentToolName::new("read_bible_neighborhood").unwrap(),
                    kind: AgentToolKind::GraphRead,
                    description: "Read a bounded bible graph neighborhood".to_string(),
                }],
            },
            budget: AgentToolBudget::default(),
            policy: AgentWorkflowPolicy::default(),
        };

        workflow.validate().unwrap();
        let encoded = serde_json::to_string(&workflow).unwrap();
        let decoded: AgentWorkflowDefinition = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, workflow);
    }

    #[test]
    fn manifest_rejects_unknown_tool_calls() {
        let manifest = AgentToolManifest { tools: Vec::new() };
        let request = AgentToolRequest {
            tool_name: AgentToolName::new("read_bible_node").unwrap(),
            arguments: AgentToolArguments::ReadBibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            },
        };

        assert_eq!(
            manifest.validate_call(&request, &AgentToolBudget::default()),
            Err(AgentWorkflowContractError::ToolNotAllowed {
                tool_name: request.tool_name
            })
        );
    }

    #[test]
    fn manifest_rejects_tool_kind_mismatch() {
        let tool_name = AgentToolName::new("read_bible_node").unwrap();
        let manifest = AgentToolManifest {
            tools: vec![AgentToolDefinition {
                name: tool_name.clone(),
                kind: AgentToolKind::GraphProposal,
                description: "wrong kind".to_string(),
            }],
        };
        let request = AgentToolRequest {
            tool_name: tool_name.clone(),
            arguments: AgentToolArguments::ReadBibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            },
        };

        assert_eq!(
            manifest.validate_call(&request, &AgentToolBudget::default()),
            Err(AgentWorkflowContractError::ToolKindMismatch {
                tool_name,
                expected: AgentToolKind::GraphProposal,
                actual: AgentToolKind::GraphRead
            })
        );
    }

    #[test]
    fn manifest_rejects_unbounded_neighborhood_reads() {
        let tool_name = AgentToolName::new("read_bible_neighborhood").unwrap();
        let manifest = AgentToolManifest {
            tools: vec![AgentToolDefinition {
                name: tool_name.clone(),
                kind: AgentToolKind::GraphRead,
                description: "Read bounded graph context".to_string(),
            }],
        };
        let budget = AgentToolBudget {
            max_graph_read_limit: 16,
            max_neighborhood_depth: 2,
            ..AgentToolBudget::default()
        };
        let request = AgentToolRequest {
            tool_name,
            arguments: AgentToolArguments::ReadBibleNeighborhood {
                node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
                depth: 3,
                limit: 16,
            },
        };

        assert_eq!(
            manifest.validate_call(&request, &budget),
            Err(AgentWorkflowContractError::ReadOutOfBounds {
                field: "depth",
                requested: 3,
                max: 2
            })
        );
    }

    #[test]
    fn duplicate_manifest_tools_are_rejected() {
        let tool_name = AgentToolName::new("read_bible_node").unwrap();
        let manifest = AgentToolManifest {
            tools: vec![
                AgentToolDefinition {
                    name: tool_name.clone(),
                    kind: AgentToolKind::GraphRead,
                    description: "Read bible node".to_string(),
                },
                AgentToolDefinition {
                    name: tool_name.clone(),
                    kind: AgentToolKind::GraphRead,
                    description: "Duplicate read bible node".to_string(),
                },
            ],
        };

        assert_eq!(
            manifest.validate(),
            Err(AgentWorkflowContractError::DuplicateTool { tool_name })
        );
    }
}
