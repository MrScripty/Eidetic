use super::*;
use crate::agent_premise_workflow::{
    ContextRefinementWorkflowKind, run_context_refinement_workflow,
};
use crate::agent_structured_tool_provider::{
    AgentStructuredTextProvider, StructuredToolLoopProvider, StructuredToolPrompt,
};
use crate::agent_workflow_harness::{AgentHarnessClock, AgentHarnessError};
use eidetic_core::contracts::{AgentRunStatus, AgentToolName, ContextStackProjection};

// Explicit synthetic structured provider; it requests evidence and verifies
// that the next turn receives exact receipts. It never interprets world facts.
struct ContextReader {
    target: eidetic_core::timeline::node::NodeId,
    turns: Vec<String>,
}

impl AgentStructuredTextProvider for ContextReader {
    fn generate_structured_tool_turn(
        &mut self,
        prompt: StructuredToolPrompt<'_>,
    ) -> Result<String, AgentHarnessError> {
        self.turns.push(prompt.prompt_text.to_string());
        if self.turns.len() == 1 {
            assert!(
                prompt
                    .workflow
                    .manifest
                    .tools
                    .iter()
                    .any(|tool| tool.name.as_str() == "read_context_stack")
            );
            Ok(
                serde_json::json!({"status":"tool_call","request":AgentToolRequest {
                    tool_name: AgentToolName::new("read_context_stack").unwrap(),
                    arguments: AgentToolArguments::ReadContextStack { target_node_id: self.target },
                }})
                .to_string(),
            )
        } else {
            Ok(
                r#"{"status":"complete","summary":"Synthetic evidence read complete; no rewrite"}"#
                    .into(),
            )
        }
    }
}

#[test]
fn actual_structured_agent_read_receives_manual_screenplay_and_keeps_historical_tool_results() {
    let (mut conn, _, a, b) = crate::timeline_script_placement::tests::fixture();
    let target = eidetic_core::timeline::node::NodeId(
        uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap(),
    );
    let first_text = "  Manual A carries a BLUE umbrella — 雨\n\n";
    crate::script_impact_review::tests::edit(&mut conn, &a, first_text);
    let mut provider = StructuredToolLoopProvider::new(ContextReader {
        target,
        turns: vec![],
    });
    let first = run_context_refinement_workflow(
        &mut conn,
        &mut provider,
        &mut AgentHarnessClock::new(100),
        ContextRefinementWorkflowKind::Scene,
    )
    .unwrap();
    assert_eq!(first.run.status, AgentRunStatus::Completed);
    assert_eq!(first.results.len(), 1);
    let AgentToolResultPayload::Text { text } = &first.results[0].payload else {
        panic!("Expected exact serialized context evidence")
    };
    let snapshot: ContextStackProjection = serde_json::from_str(text).unwrap();
    let input = snapshot
        .script_context
        .unwrap()
        .into_iter()
        .find(|input| input.block_id == a.block_id)
        .unwrap();
    assert_eq!(input.text, first_text);
    let second_turn = &provider.into_inner().turns[1];
    // Provider sees serialized completed tool results, including serialized
    // evidence JSON; parse history separately for exact whitespace assertions.
    assert!(second_turn.contains("BLUE umbrella"));
    assert!(second_turn.contains(&input.revision_event_id.0.to_string()));
    let old_result = first.results[0].clone();
    crate::script_impact_review::tests::edit(&mut conn, &a, "Changed after the historical read");
    crate::script_impact_review::tests::edit(&mut conn, &a, first_text);
    let mut provider = StructuredToolLoopProvider::new(ContextReader {
        target,
        turns: vec![],
    });
    let next = run_context_refinement_workflow(
        &mut conn,
        &mut provider,
        &mut AgentHarnessClock::new(200),
        ContextRefinementWorkflowKind::Scene,
    )
    .unwrap();
    let AgentToolResultPayload::Text { text } = &next.results[0].payload else {
        panic!("Expected context evidence")
    };
    let current: ContextStackProjection = serde_json::from_str(text).unwrap();
    let current_input = current
        .script_context
        .unwrap()
        .into_iter()
        .find(|entry| entry.block_id == a.block_id)
        .unwrap();
    assert_eq!(current_input.text, first_text);
    assert_ne!(current_input.revision_event_id, input.revision_event_id);
    assert_eq!(
        crate::agent_workflow_store::load_agent_tool_result(&conn, old_result.call_id)
            .unwrap()
            .unwrap(),
        old_result
    );
    assert_eq!(crate::script_impact_review::tests::text(&conn, &b), b.text);
    let proposals = graph_proposal_store::load_graph_proposal_list_projection(&conn).unwrap();
    assert!(proposals.payload.proposals.is_empty());
}

#[test]
fn agent_context_read_of_missing_target_reports_failure_instead_of_empty_success() {
    let (mut conn, _, _, _) = crate::timeline_script_placement::tests::fixture();
    let target = eidetic_core::timeline::node::NodeId::new();
    let req = AgentToolRequest {
        tool_name: AgentToolName::new("read_context_stack").unwrap(),
        arguments: AgentToolArguments::ReadContextStack {
            target_node_id: target,
        },
    };
    assert!(AgentGraphReadTools::new(&conn).execute_tool(&req).is_err());
    let mut provider = StructuredToolLoopProvider::new(ContextReader {
        target,
        turns: vec![],
    });
    assert!(
        run_context_refinement_workflow(
            &mut conn,
            &mut provider,
            &mut AgentHarnessClock::new(100),
            ContextRefinementWorkflowKind::Scene
        )
        .is_err()
    );
    let status: String = conn
        .query_row(
            "SELECT status FROM agent_runs ORDER BY rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "failed");
}
