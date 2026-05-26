use eidetic_core::contracts::{
    AgentToolArguments, AgentToolKind, AgentToolRequest, AgentToolResultPayload,
    BibleGraphNodeListProjection, BibleRenderGraphProjectionRequest, CommandEnvelope,
    ContextStackProjection, CreateGraphProposalCommand, GraphProposalAction, GraphProposalId,
    GraphProposalTarget,
};
use eidetic_core::contracts::{BibleGraphEdgeId, BibleGraphNodeId};
use rusqlite::Connection;

use crate::agent_workflow_harness::{AgentHarnessError, AgentWorkflowToolExecutor};
use crate::bible_graph_store;
use crate::context_influence_store;
use crate::graph_proposal_store;
use crate::timeline_node_store;

pub struct AgentGraphReadTools<'a> {
    conn: &'a Connection,
}

impl<'a> AgentGraphReadTools<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl AgentWorkflowToolExecutor for AgentGraphReadTools<'_> {
    fn execute_tool(
        &mut self,
        request: &AgentToolRequest,
    ) -> Result<AgentToolResultPayload, AgentHarnessError> {
        match &request.arguments {
            AgentToolArguments::SearchBibleNodes { query, limit } => {
                let projection = search_bible_nodes(self.conn, query, *limit)?;
                json_payload(&projection)
            }
            AgentToolArguments::ReadBibleNode { node_id } => {
                let projection = bible_graph_store::load_node_list_projection(self.conn)?;
                let node = projection
                    .nodes
                    .into_iter()
                    .find(|node| node.id == *node_id)
                    .ok_or_else(|| {
                        AgentHarnessError::Tool(format!(
                            "bible node not found: {}",
                            node_id.as_str()
                        ))
                    })?;
                json_payload(&node)
            }
            AgentToolArguments::ReadBibleNeighborhood {
                node_id,
                depth,
                limit,
            } => {
                bible_graph_store::create_schema(self.conn)?;
                let projection = bible_graph_store::load_render_graph_projection(
                    self.conn,
                    &BibleRenderGraphProjectionRequest {
                        selected_node_id: Some(node_id.clone()),
                        neighborhood_depth: u32::from(*depth),
                        max_nodes: *limit,
                        ..BibleRenderGraphProjectionRequest::default()
                    },
                )?;
                json_payload(&projection)
            }
            AgentToolArguments::ReadContextStack { target_node_id } => {
                let nodes =
                    timeline_node_store::load_node_ancestor_stack(self.conn, *target_node_id)?;
                let projection = ContextStackProjection::from_nodes(&nodes, *target_node_id)
                    .ok_or_else(|| {
                        AgentHarnessError::Tool(format!(
                            "context stack target node not found: {}",
                            target_node_id.0
                        ))
                    })?;
                json_payload(&projection)
            }
            AgentToolArguments::ReadActiveGraphContext { target_node_id } => {
                bible_graph_store::create_schema(self.conn)?;
                let projection = bible_graph_store::load_render_graph_projection(
                    self.conn,
                    &BibleRenderGraphProjectionRequest {
                        selected_timeline_node_id: Some(*target_node_id),
                        ..BibleRenderGraphProjectionRequest::default()
                    },
                )?;
                json_payload(&projection)
            }
            AgentToolArguments::ReadInfluencePaths {
                target_node_id,
                limit,
            } => {
                let mut records = context_influence_store::load_latest_context_influence_records(
                    self.conn,
                    *target_node_id,
                )?;
                records.truncate(*limit as usize);
                json_payload(&records)
            }
            AgentToolArguments::ProposeBibleNode { .. }
            | AgentToolArguments::ProposeBibleField { .. }
            | AgentToolArguments::ProposeBibleEdge { .. }
            | AgentToolArguments::ProposeTimelineContextLink { .. }
            | AgentToolArguments::RecordContextEvaluation { .. } => Err(AgentHarnessError::Tool(
                "agent graph read tools do not execute write/proposal tools".to_string(),
            )),
        }
    }
}

pub struct AgentGraphProposalTools<'a> {
    conn: &'a mut Connection,
}

impl<'a> AgentGraphProposalTools<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }
}

impl AgentWorkflowToolExecutor for AgentGraphProposalTools<'_> {
    fn execute_tool(
        &mut self,
        request: &AgentToolRequest,
    ) -> Result<AgentToolResultPayload, AgentHarnessError> {
        let command = match &request.arguments {
            AgentToolArguments::ProposeBibleNode {
                command_id,
                parent_id,
                schema_key,
                title,
                summary,
            } => CreateGraphProposalCommand {
                proposal_id: proposal_id("node", command_id),
                action: GraphProposalAction::CreateBibleNode,
                target: GraphProposalTarget::BibleNode {
                    node_id: proposed_node_id(command_id),
                    parent_id: parent_id.clone(),
                    schema_key: schema_key.clone(),
                    title: title.clone(),
                },
                summary: summary.clone(),
                proposed_value: None,
                rationale: Some(summary.clone()),
                source_agent_run_id: None,
                source_tool_call_id: None,
            },
            AgentToolArguments::ProposeBibleField {
                command_id,
                node_id,
                part_key,
                field_key,
                value,
            } => CreateGraphProposalCommand {
                proposal_id: proposal_id("field", command_id),
                action: GraphProposalAction::SetBibleField,
                target: GraphProposalTarget::BibleField {
                    node_id: node_id.clone(),
                    part_key: part_key.clone(),
                    field_key: field_key.clone(),
                    field_id: None,
                },
                summary: format!("Set {}.{}", part_key.as_str(), field_key.as_str()),
                proposed_value: Some(eidetic_core::contracts::FieldValue::Text(value.clone())),
                rationale: Some(value.clone()),
                source_agent_run_id: None,
                source_tool_call_id: None,
            },
            AgentToolArguments::ProposeBibleEdge {
                command_id,
                from_node_id,
                to_node_id,
                edge_kind,
                label,
            } => CreateGraphProposalCommand {
                proposal_id: proposal_id("edge", command_id),
                action: GraphProposalAction::CreateBibleEdge,
                target: GraphProposalTarget::BibleEdge {
                    edge_id: proposed_edge_id(command_id),
                    from_node_id: from_node_id.clone(),
                    to_node_id: to_node_id.clone(),
                    edge_kind: edge_kind.clone(),
                    label: label.clone(),
                },
                summary: format!("Create graph edge {label}"),
                proposed_value: None,
                rationale: Some(label.clone()),
                source_agent_run_id: None,
                source_tool_call_id: None,
            },
            AgentToolArguments::ProposeTimelineContextLink {
                command_id,
                timeline_node_id,
                bible_node_id,
                rationale,
            } => CreateGraphProposalCommand {
                proposal_id: proposal_id("context_link", command_id),
                action: GraphProposalAction::LinkTimelineContext,
                target: GraphProposalTarget::TimelineContextLink {
                    timeline_node_id: *timeline_node_id,
                    bible_node_id: bible_node_id.clone(),
                },
                summary: "Link bible node to timeline context".to_string(),
                proposed_value: None,
                rationale: Some(rationale.clone()),
                source_agent_run_id: None,
                source_tool_call_id: None,
            },
            AgentToolArguments::SearchBibleNodes { .. }
            | AgentToolArguments::ReadBibleNode { .. }
            | AgentToolArguments::ReadBibleNeighborhood { .. }
            | AgentToolArguments::ReadContextStack { .. }
            | AgentToolArguments::ReadActiveGraphContext { .. }
            | AgentToolArguments::ReadInfluencePaths { .. }
            | AgentToolArguments::RecordContextEvaluation { .. } => {
                return Err(AgentHarnessError::Tool(
                    "agent graph proposal tools only execute proposal tools".to_string(),
                ));
            }
        };

        graph_proposal_store::record_create_graph_proposal(
            self.conn,
            &CommandEnvelope {
                id: request_command_id(&request.arguments),
                payload: command,
            },
            0,
        )?;
        let projection = graph_proposal_store::load_graph_proposal_list_projection(self.conn)?;
        json_payload(&projection.payload)
    }
}

pub struct AgentGraphWorkflowTools<'a> {
    conn: &'a mut Connection,
}

impl<'a> AgentGraphWorkflowTools<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }
}

impl AgentWorkflowToolExecutor for AgentGraphWorkflowTools<'_> {
    fn execute_tool(
        &mut self,
        request: &AgentToolRequest,
    ) -> Result<AgentToolResultPayload, AgentHarnessError> {
        match request.arguments.kind() {
            AgentToolKind::GraphRead => AgentGraphReadTools::new(self.conn).execute_tool(request),
            AgentToolKind::GraphProposal => {
                AgentGraphProposalTools::new(self.conn).execute_tool(request)
            }
            AgentToolKind::ContextEvaluation => Err(AgentHarnessError::Tool(
                "agent graph workflow tools do not execute context-evaluation tools".to_string(),
            )),
        }
    }
}

fn search_bible_nodes(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<BibleGraphNodeListProjection, AgentHarnessError> {
    bible_graph_store::create_schema(conn)?;
    let mut projection = bible_graph_store::load_node_list_projection(conn)?;
    let query = query.trim().to_ascii_lowercase();
    projection.nodes.retain(|node| {
        node.id.as_str().to_ascii_lowercase().contains(&query)
            || node.name.to_ascii_lowercase().contains(&query)
            || node
                .schema_key
                .as_str()
                .to_ascii_lowercase()
                .contains(&query)
    });
    projection.nodes.truncate(limit as usize);
    Ok(projection)
}

fn json_payload<T: serde::Serialize>(
    value: &T,
) -> Result<AgentToolResultPayload, AgentHarnessError> {
    Ok(AgentToolResultPayload::Text {
        text: serde_json::to_string(value)
            .map_err(|error| AgentHarnessError::Tool(error.to_string()))?,
    })
}

fn request_command_id(arguments: &AgentToolArguments) -> eidetic_core::contracts::CommandId {
    match arguments {
        AgentToolArguments::ProposeBibleNode { command_id, .. }
        | AgentToolArguments::ProposeBibleField { command_id, .. }
        | AgentToolArguments::ProposeBibleEdge { command_id, .. }
        | AgentToolArguments::ProposeTimelineContextLink { command_id, .. }
        | AgentToolArguments::RecordContextEvaluation { command_id, .. } => *command_id,
        AgentToolArguments::SearchBibleNodes { .. }
        | AgentToolArguments::ReadBibleNode { .. }
        | AgentToolArguments::ReadBibleNeighborhood { .. }
        | AgentToolArguments::ReadContextStack { .. }
        | AgentToolArguments::ReadActiveGraphContext { .. }
        | AgentToolArguments::ReadInfluencePaths { .. } => {
            eidetic_core::contracts::CommandId::new()
        }
    }
}

fn proposal_id(kind: &str, command_id: &eidetic_core::contracts::CommandId) -> GraphProposalId {
    GraphProposalId::new(format!("proposal.agent.{kind}.{}", command_id.0))
        .expect("generated proposal ids are non-empty")
}

fn proposed_node_id(command_id: &eidetic_core::contracts::CommandId) -> BibleGraphNodeId {
    BibleGraphNodeId::new(format!("proposal.node.{}", command_id.0))
        .expect("generated node ids are non-empty")
}

fn proposed_edge_id(command_id: &eidetic_core::contracts::CommandId) -> BibleGraphEdgeId {
    BibleGraphEdgeId::new(format!("proposal.edge.{}", command_id.0))
        .expect("generated edge ids are non-empty")
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        AgentToolName, AgentToolRequest, BibleGraphEdgeKind, BibleGraphNodeId, CommandEnvelope,
        CreateBibleGraphNodeCommand, GraphProposalListProjection,
    };

    #[test]
    fn graph_read_tools_search_bible_nodes_without_mutating_state() {
        let mut conn = Connection::open_in_memory().unwrap();
        bible_graph_store::create_schema(&conn).unwrap();
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let command = CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: node_id.clone(),
            parent_id: Some(BibleGraphNodeId::new("canonical.characters").unwrap()),
            schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("canonical.character")
                .unwrap(),
            name: "Ada".to_string(),
            sort_order: 1,
        });
        crate::bible_graph_command::apply_ensure_canonical_bible_roots(
            &mut conn,
            &CommandEnvelope::new(eidetic_core::contracts::EnsureCanonicalBibleRootsCommand {}),
            1,
        )
        .unwrap();
        crate::bible_graph_command::apply_create_bible_graph_node(&mut conn, &command, 10).unwrap();
        let mut tools = AgentGraphReadTools::new(&conn);

        let payload = tools
            .execute_tool(&AgentToolRequest {
                tool_name: AgentToolName::new("search_bible_nodes").unwrap(),
                arguments: AgentToolArguments::SearchBibleNodes {
                    query: "Ada".to_string(),
                    limit: 4,
                },
            })
            .unwrap();

        let AgentToolResultPayload::Text { text } = payload else {
            panic!("expected text payload");
        };
        let projection: BibleGraphNodeListProjection = serde_json::from_str(&text).unwrap();
        assert_eq!(projection.nodes.len(), 1);
        assert_eq!(projection.nodes[0].id, node_id);
    }

    #[test]
    fn graph_read_tools_return_empty_bounded_neighborhood_for_empty_graph() {
        let conn = Connection::open_in_memory().unwrap();
        let mut tools = AgentGraphReadTools::new(&conn);

        let payload = tools
            .execute_tool(&AgentToolRequest {
                tool_name: AgentToolName::new("read_bible_neighborhood").unwrap(),
                arguments: AgentToolArguments::ReadBibleNeighborhood {
                    node_id: BibleGraphNodeId::new("node.missing").unwrap(),
                    depth: 1,
                    limit: 8,
                },
            })
            .unwrap();

        let AgentToolResultPayload::Text { text } = payload else {
            panic!("expected text payload");
        };
        assert!(text.contains("\"nodes\":[]"));
    }

    #[test]
    fn graph_read_tools_reject_write_tool_execution() {
        let conn = Connection::open_in_memory().unwrap();
        let mut tools = AgentGraphReadTools::new(&conn);

        let error = tools
            .execute_tool(&AgentToolRequest {
                tool_name: AgentToolName::new("propose_bible_node").unwrap(),
                arguments: AgentToolArguments::ProposeBibleNode {
                    command_id: eidetic_core::contracts::CommandId::new(),
                    parent_id: BibleGraphNodeId::new("canonical.characters").unwrap(),
                    schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new(
                        "canonical.character",
                    )
                    .unwrap(),
                    title: "Ada".to_string(),
                    summary: "A character".to_string(),
                },
            })
            .unwrap_err();

        assert!(matches!(error, AgentHarnessError::Tool(_)));
    }

    #[test]
    fn graph_proposal_tools_record_node_proposals_without_mutating_graph() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command_id = eidetic_core::contracts::CommandId::new();
        let mut tools = AgentGraphProposalTools::new(&mut conn);

        let payload = tools
            .execute_tool(&AgentToolRequest {
                tool_name: AgentToolName::new("propose_bible_node").unwrap(),
                arguments: AgentToolArguments::ProposeBibleNode {
                    command_id,
                    parent_id: BibleGraphNodeId::new("canonical.characters").unwrap(),
                    schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new(
                        "canonical.character",
                    )
                    .unwrap(),
                    title: "Ada".to_string(),
                    summary: "Create Ada".to_string(),
                },
            })
            .unwrap();

        let AgentToolResultPayload::Text { text } = payload else {
            panic!("expected text payload");
        };
        let projection: GraphProposalListProjection = serde_json::from_str(&text).unwrap();
        assert_eq!(projection.proposals.len(), 1);
        assert_eq!(
            projection.proposals[0].id.as_str(),
            format!("proposal.agent.node.{}", command_id.0)
        );
        let graph_node_table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'bible_graph_nodes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(graph_node_table_count, 0);
    }

    #[test]
    fn graph_proposal_tools_record_edge_kind_from_tool_arguments() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command_id = eidetic_core::contracts::CommandId::new();
        let mut tools = AgentGraphProposalTools::new(&mut conn);

        let payload = tools
            .execute_tool(&AgentToolRequest {
                tool_name: AgentToolName::new("propose_bible_edge").unwrap(),
                arguments: AgentToolArguments::ProposeBibleEdge {
                    command_id,
                    from_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
                    to_node_id: BibleGraphNodeId::new("node.place.observatory").unwrap(),
                    edge_kind: BibleGraphEdgeKind::LocatedIn,
                    label: "Ada appears in the observatory".to_string(),
                },
            })
            .unwrap();

        let AgentToolResultPayload::Text { text } = payload else {
            panic!("expected text payload");
        };
        let projection: GraphProposalListProjection = serde_json::from_str(&text).unwrap();
        assert_eq!(projection.proposals.len(), 1);
        let eidetic_core::contracts::GraphProposalTarget::BibleEdge { edge_kind, .. } =
            &projection.proposals[0].target
        else {
            panic!("expected bible edge proposal");
        };
        assert_eq!(edge_kind, &BibleGraphEdgeKind::LocatedIn);
    }

    #[test]
    fn graph_proposal_tools_reject_read_tool_execution() {
        let mut conn = Connection::open_in_memory().unwrap();
        let mut tools = AgentGraphProposalTools::new(&mut conn);

        let error = tools
            .execute_tool(&AgentToolRequest {
                tool_name: AgentToolName::new("search_bible_nodes").unwrap(),
                arguments: AgentToolArguments::SearchBibleNodes {
                    query: "Ada".to_string(),
                    limit: 4,
                },
            })
            .unwrap_err();

        assert!(matches!(error, AgentHarnessError::Tool(_)));
    }
}
