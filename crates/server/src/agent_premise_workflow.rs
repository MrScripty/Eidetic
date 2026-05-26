use eidetic_core::contracts::{
    AgentToolBudget, AgentToolDefinition, AgentToolKind, AgentToolManifest, AgentToolName,
    AgentWorkflowDefinition, AgentWorkflowId, AgentWorkflowIntent, AgentWorkflowPolicy,
};
use rusqlite::Connection;

use crate::agent_graph_tools::AgentGraphWorkflowTools;
use crate::agent_workflow_harness::{
    AgentHarnessClock, AgentHarnessError, AgentWorkflowProvider, AgentWorkflowToolExecutor,
    run_agent_workflow_with_connection_tools,
};
use crate::agent_workflow_service::AgentRunHistoryProjection;

pub fn premise_graph_context_workflow() -> AgentWorkflowDefinition {
    AgentWorkflowDefinition {
        id: AgentWorkflowId::new("workflow.premise.graph_context")
            .expect("static workflow id is non-empty"),
        label: "Premise graph context".to_string(),
        intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
        manifest: AgentToolManifest {
            tools: vec![
                AgentToolDefinition {
                    name: AgentToolName::new("read_active_graph_context")
                        .expect("static tool name is non-empty"),
                    kind: AgentToolKind::GraphRead,
                    description: "Read the active graph context for the selected timeline node"
                        .to_string(),
                },
                AgentToolDefinition {
                    name: AgentToolName::new("propose_bible_node")
                        .expect("static tool name is non-empty"),
                    kind: AgentToolKind::GraphProposal,
                    description: "Propose a reviewable story-bible node".to_string(),
                },
                AgentToolDefinition {
                    name: AgentToolName::new("propose_bible_edge")
                        .expect("static tool name is non-empty"),
                    kind: AgentToolKind::GraphProposal,
                    description: "Propose a reviewable story-bible edge".to_string(),
                },
            ],
        },
        budget: AgentToolBudget {
            max_tool_calls: 4,
            ..AgentToolBudget::default()
        },
        policy: AgentWorkflowPolicy::default(),
    }
}

pub fn run_premise_graph_context_workflow<P>(
    conn: &mut Connection,
    provider: &mut P,
    clock: &mut AgentHarnessClock,
) -> Result<AgentRunHistoryProjection, AgentHarnessError>
where
    P: AgentWorkflowProvider,
{
    let workflow = premise_graph_context_workflow();
    run_agent_workflow_with_connection_tools(
        conn,
        workflow,
        provider,
        |conn, request| AgentGraphWorkflowTools::new(conn).execute_tool(request),
        clock,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_structured_tool_provider::{
        AgentStructuredTextProvider, StructuredToolLoopProvider, StructuredToolPrompt,
    };
    use crate::agent_workflow_harness::AgentHarnessError;
    use crate::graph_proposal_store;
    use eidetic_core::contracts::{
        AgentRunStatus, AgentToolArguments, AgentToolRequest, AgentToolResultPayload,
        BibleGraphNodeId, BibleGraphSchemaKey, CommandId,
    };
    use eidetic_core::timeline::node::NodeId;

    #[test]
    fn premise_workflow_reads_context_and_records_reviewable_graph_proposal() {
        let mut conn = Connection::open_in_memory().unwrap();
        let timeline_node_id = NodeId::new();
        let command_id = CommandId::new();
        let read_context = serde_json::json!({
            "status": "tool_call",
            "request": AgentToolRequest {
                tool_name: AgentToolName::new("read_active_graph_context").unwrap(),
                arguments: AgentToolArguments::ReadActiveGraphContext {
                    target_node_id: timeline_node_id,
                },
            },
        })
        .to_string();
        let propose_node = serde_json::json!({
            "status": "tool_call",
            "request": AgentToolRequest {
                tool_name: AgentToolName::new("propose_bible_node").unwrap(),
                arguments: AgentToolArguments::ProposeBibleNode {
                    command_id,
                    parent_id: BibleGraphNodeId::new("canonical.characters").unwrap(),
                    schema_key: BibleGraphSchemaKey::new("canonical.character").unwrap(),
                    title: "Ada".to_string(),
                    summary: "Premise introduces Ada".to_string(),
                },
            },
        })
        .to_string();
        let mut provider = StructuredToolLoopProvider::new(QueuedTextProvider {
            responses: vec![
                r#"{"status":"complete","summary":"proposal recorded"}"#.to_string(),
                propose_node,
                read_context,
            ],
        });
        let mut clock = AgentHarnessClock::new(100);

        let history =
            run_premise_graph_context_workflow(&mut conn, &mut provider, &mut clock).unwrap();

        assert_eq!(history.run.status, AgentRunStatus::Completed);
        assert_eq!(history.calls.len(), 2);
        assert_eq!(history.results.len(), 2);
        assert!(matches!(
            history.results[0].payload,
            AgentToolResultPayload::Text { .. }
        ));
        let proposals = graph_proposal_store::load_graph_proposal_list_projection(&conn).unwrap();
        assert_eq!(proposals.payload.proposals.len(), 1);
        assert_eq!(
            proposals.payload.proposals[0].id.as_str(),
            format!("proposal.agent.node.{}", command_id.0)
        );
    }

    struct QueuedTextProvider {
        responses: Vec<String>,
    }

    impl AgentStructuredTextProvider for QueuedTextProvider {
        fn generate_structured_tool_turn(
            &mut self,
            _prompt: StructuredToolPrompt<'_>,
        ) -> Result<String, AgentHarnessError> {
            self.responses.pop().ok_or_else(|| {
                AgentHarnessError::Provider("missing structured test response".to_string())
            })
        }
    }
}
