use eidetic_core::contracts::{
    AgentToolArguments, AgentToolRequest, AgentToolResultPayload, BibleGraphNodeListProjection,
    BibleRenderGraphProjectionRequest, ContextStackProjection,
};
use rusqlite::Connection;

use crate::agent_workflow_harness::{AgentHarnessError, AgentWorkflowToolExecutor};
use crate::bible_graph_store;
use crate::context_influence_store;
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

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        AgentToolName, AgentToolRequest, BibleGraphNodeId, CommandEnvelope,
        CreateBibleGraphNodeCommand,
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
}
