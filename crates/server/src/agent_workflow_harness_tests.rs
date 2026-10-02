use super::*;
use eidetic_core::contracts::{
    AgentToolArguments, AgentToolBudget, AgentToolDefinition, AgentToolKind, AgentToolManifest,
    AgentToolName, AgentWorkflowId, AgentWorkflowIntent, AgentWorkflowPolicy, BibleGraphNodeId,
};

#[test]
fn mock_provider_harness_records_validated_tool_history() {
    let mut conn = Connection::open_in_memory().unwrap();
    let tool_name = AgentToolName::new("read_bible_node").unwrap();
    let workflow = workflow_with_tool(tool_name.clone(), AgentToolKind::GraphRead, 4);
    let request = AgentToolRequest {
        tool_name,
        arguments: AgentToolArguments::ReadBibleNode {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        },
    };
    let mut provider = MockProvider {
        requests: vec![request],
    };
    let mut tools = MockTools;
    let mut clock = AgentHarnessClock::new(10);

    let history =
        run_mockable_agent_workflow(&mut conn, workflow, &mut provider, &mut tools, &mut clock)
            .unwrap();

    assert_eq!(history.run.status, AgentRunStatus::Completed);
    assert_eq!(history.calls.len(), 1);
    assert_eq!(history.calls[0].status, AgentToolCallStatus::Completed);
    assert_eq!(history.results.len(), 1);
    assert_eq!(history.results[0].status, AgentToolResultStatus::Succeeded);
    assert_eq!(
        history.results[0].payload,
        AgentToolResultPayload::Text {
            text: "mock tool result".to_string()
        }
    );
}

#[test]
fn mock_provider_harness_rejects_disallowed_tool_calls_before_execution() {
    let mut conn = Connection::open_in_memory().unwrap();
    let workflow = workflow_with_tool(
        AgentToolName::new("read_bible_node").unwrap(),
        AgentToolKind::GraphRead,
        4,
    );
    let request = AgentToolRequest {
        tool_name: AgentToolName::new("propose_bible_node").unwrap(),
        arguments: AgentToolArguments::ProposeBibleNode {
            command_id: eidetic_core::contracts::CommandId::new(),
            parent_id: BibleGraphNodeId::new("canon.characters").unwrap(),
            schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("canonical.character")
                .unwrap(),
            title: "Ada".to_string(),
            summary: "Premise character".to_string(),
        },
    };
    let mut provider = MockProvider {
        requests: vec![request],
    };
    let mut clock = AgentHarnessClock::new(10);

    let error = run_agent_workflow_with_connection_tools(
        &mut conn,
        workflow,
        &mut provider,
        |_, _| panic!("rejected manifest call must not execute"),
        &mut clock,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        AgentHarnessError::Contract(
            eidetic_core::contracts::AgentWorkflowContractError::ToolNotAllowed { .. }
        )
    ));
    let history = only_history(&conn);
    assert_eq!(history.run.status, AgentRunStatus::Failed);
    assert!(history.run.completed_at_ms.is_some());
    assert_eq!(history.calls[0].status, AgentToolCallStatus::Rejected);
    assert_eq!(history.results[0].status, AgentToolResultStatus::Rejected);
}

#[test]
fn mock_provider_harness_fails_closed_on_tool_budget_exhaustion() {
    let mut conn = Connection::open_in_memory().unwrap();
    let tool_name = AgentToolName::new("read_bible_node").unwrap();
    let workflow = workflow_with_tool(tool_name.clone(), AgentToolKind::GraphRead, 1);
    let request = AgentToolRequest {
        tool_name,
        arguments: AgentToolArguments::ReadBibleNode {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        },
    };
    let mut provider = MockProvider {
        requests: vec![request.clone(), request],
    };
    let mut tools = MockTools;
    let mut clock = AgentHarnessClock::new(10);

    let error =
        run_mockable_agent_workflow(&mut conn, workflow, &mut provider, &mut tools, &mut clock)
            .unwrap_err();

    assert!(matches!(
        error,
        AgentHarnessError::BudgetExceeded { max_tool_calls: 1 }
    ));
    let history = only_history(&conn);
    assert_eq!(history.run.status, AgentRunStatus::Failed);
    assert_eq!(history.calls.len(), 1);
}

#[test]
fn completion_after_exact_tool_budget_is_allowed() {
    let mut conn = Connection::open_in_memory().unwrap();
    let (workflow, request) = one_read_workflow();
    let mut provider = MockProvider {
        requests: vec![request],
    };
    let history = run_mockable_agent_workflow(
        &mut conn,
        workflow,
        &mut provider,
        &mut MockTools,
        &mut AgentHarnessClock::new(10),
    )
    .unwrap();
    assert_eq!(history.run.status, AgentRunStatus::Completed);
    assert_eq!(history.calls.len(), 1);
}

#[test]
fn provider_failure_preserves_completed_tools_and_marks_run_failed() {
    struct FailingProvider(Option<AgentToolRequest>);
    impl AgentWorkflowProvider for FailingProvider {
        fn next_tool_request(
            &mut self,
            _: AgentProviderTurn<'_>,
        ) -> Result<Option<AgentToolRequest>, AgentHarnessError> {
            self.0
                .take()
                .map(Some)
                .ok_or_else(|| AgentHarnessError::Provider("offline".into()))
        }
    }
    let mut conn = Connection::open_in_memory().unwrap();
    let (workflow, request) = one_read_workflow();
    let error = run_mockable_agent_workflow(
        &mut conn,
        workflow,
        &mut FailingProvider(Some(request)),
        &mut MockTools,
        &mut AgentHarnessClock::new(10),
    )
    .unwrap_err();
    assert!(matches!(error, AgentHarnessError::Provider(_)));
    let history = only_history(&conn);
    assert_eq!(history.run.status, AgentRunStatus::Failed);
    assert_eq!(history.run.error.as_deref(), Some("offline"));
    assert!(history.run.completed_at_ms.is_some());
    assert_eq!(history.calls[0].status, AgentToolCallStatus::Completed);
    assert_eq!(history.results[0].status, AgentToolResultStatus::Succeeded);
}

#[test]
fn failing_tool_has_persisted_intent_and_failure_without_retry() {
    let mut conn = Connection::open_in_memory().unwrap();
    let (workflow, request) = one_read_workflow();
    let mut provider = MockProvider {
        requests: vec![request],
    };
    let mut executions = 0;
    let error = run_agent_workflow_with_connection_tools(
        &mut conn,
        workflow,
        &mut provider,
        |conn, _| {
            executions += 1;
            let history = only_history(conn);
            assert_eq!(history.calls[0].status, AgentToolCallStatus::Running);
            assert!(history.results.is_empty());
            Err(AgentHarnessError::Tool("stale source revision".into()))
        },
        &mut AgentHarnessClock::new(10),
    )
    .unwrap_err();
    assert!(matches!(error, AgentHarnessError::Tool(_)));
    assert_eq!(executions, 1);
    let history = only_history(&conn);
    assert_eq!(history.run.status, AgentRunStatus::Failed);
    assert_eq!(history.calls[0].status, AgentToolCallStatus::Failed);
    assert_eq!(history.results[0].status, AgentToolResultStatus::Failed);
    assert_eq!(
        history.results[0].payload,
        AgentToolResultPayload::Error {
            message: "stale source revision".into(),
        }
    );
}

fn one_read_workflow() -> (AgentWorkflowDefinition, AgentToolRequest) {
    let tool_name = AgentToolName::new("read_bible_node").unwrap();
    let workflow = workflow_with_tool(tool_name.clone(), AgentToolKind::GraphRead, 1);
    (
        workflow,
        AgentToolRequest {
            tool_name,
            arguments: AgentToolArguments::ReadBibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            },
        },
    )
}

fn only_history(conn: &Connection) -> AgentRunHistoryProjection {
    let id: String = conn
        .query_row("SELECT id FROM agent_runs", [], |row| row.get(0))
        .unwrap();
    load_history(conn, AgentRunId(uuid::Uuid::parse_str(&id).unwrap())).unwrap()
}

#[test]
fn cooperative_cancellation_is_terminal_and_does_not_execute_tools() {
    struct CancelledProvider;
    impl AgentWorkflowProvider for CancelledProvider {
        fn next_tool_request(
            &mut self,
            _: AgentProviderTurn<'_>,
        ) -> Result<Option<AgentToolRequest>, AgentHarnessError> {
            Err(AgentHarnessError::Cancelled)
        }
    }
    let mut conn = Connection::open_in_memory().unwrap();
    let (workflow, _) = one_read_workflow();
    let error = run_agent_workflow_with_connection_tools(
        &mut conn,
        workflow,
        &mut CancelledProvider,
        |_, _| panic!("cancelled workflow must not execute tools"),
        &mut AgentHarnessClock::new(10),
    )
    .unwrap_err();
    assert!(matches!(error, AgentHarnessError::Cancelled));
    let history = only_history(&conn);
    assert_eq!(history.run.status, AgentRunStatus::Cancelled);
    assert!(history.run.completed_at_ms.is_some());
    assert!(history.calls.is_empty());
}

#[test]
fn terminal_persistence_failure_reports_original_execution_error() {
    let mut conn = Connection::open_in_memory().unwrap();
    let (workflow, request) = one_read_workflow();
    let mut provider = MockProvider {
        requests: vec![request],
    };
    let error = run_agent_workflow_with_connection_tools(
        &mut conn,
        workflow,
        &mut provider,
        |conn, _| {
            conn.execute_batch(
                "CREATE TRIGGER reject_run_finish BEFORE UPDATE ON agent_runs
                BEGIN SELECT RAISE(FAIL, 'history unavailable'); END;",
            )
            .unwrap();
            Err(AgentHarnessError::Tool("source changed".into()))
        },
        &mut AgentHarnessClock::new(10),
    )
    .unwrap_err();
    match error {
        AgentHarnessError::Finalization {
            execution_error,
            persistence_error,
        } => {
            assert!(
                matches!(execution_error.as_deref(), Some(AgentHarnessError::Tool(message))
                if message == "source changed")
            );
            assert!(persistence_error.contains("history unavailable"));
        }
        other => panic!("expected explicit persistence failure, got {other}"),
    }
    // The API must not claim durable terminal state when the store rejected it.
    assert_eq!(only_history(&conn).run.status, AgentRunStatus::Running);
}

#[test]
fn tool_failure_survives_call_update_and_result_insert_failures() {
    assert_terminal_write_faults(TerminalOutcome::Tool);
}

#[test]
fn cancellation_survives_call_update_and_result_insert_failures() {
    assert_terminal_write_faults(TerminalOutcome::Cancelled);
}

#[test]
fn validation_failure_survives_rejected_call_and_result_insert_failures() {
    assert_terminal_write_faults(TerminalOutcome::Rejected);
}

#[derive(Clone, Copy)]
enum TerminalOutcome {
    Tool,
    Cancelled,
    Rejected,
}

fn assert_terminal_write_faults(expected: TerminalOutcome) {
    for fail_call_write in [true, false] {
        let mut conn = Connection::open_in_memory().unwrap();
        agent_workflow_store::create_schema(&conn).unwrap();
        // Rejected calls are inserted directly as Rejected, never executed.
        let point = if fail_call_write {
            if matches!(expected, TerminalOutcome::Rejected) {
                "INSERT ON agent_tool_calls"
            } else {
                "UPDATE ON agent_tool_calls"
            }
        } else {
            "INSERT ON agent_tool_results"
        };
        conn.execute_batch(&format!(
            "CREATE TRIGGER reject_terminal_history BEFORE {point}
             BEGIN SELECT RAISE(FAIL, 'terminal history denied'); END;"
        ))
        .unwrap();
        let (mut workflow, request) = one_read_workflow();
        if matches!(expected, TerminalOutcome::Rejected) {
            workflow.manifest.tools.clear();
        }
        let mut provider = MockProvider {
            requests: vec![request],
        };
        let mut executions = 0;
        let error = run_agent_workflow_with_connection_tools(
            &mut conn,
            workflow,
            &mut provider,
            |_, _| {
                executions += 1;
                match expected {
                    TerminalOutcome::Tool => Err(AgentHarnessError::Tool("source changed".into())),
                    TerminalOutcome::Cancelled => Err(AgentHarnessError::Cancelled),
                    TerminalOutcome::Rejected => panic!("rejected call must not execute"),
                }
            },
            &mut AgentHarnessClock::new(10),
        )
        .unwrap_err();
        match error {
            AgentHarnessError::Finalization {
                execution_error,
                persistence_error,
            } => {
                assert!(persistence_error.contains("terminal history denied"));
                match (expected, execution_error.as_deref()) {
                    (TerminalOutcome::Tool, Some(AgentHarnessError::Tool(message))) => {
                        assert_eq!(message, "source changed");
                    }
                    (TerminalOutcome::Cancelled, Some(AgentHarnessError::Cancelled)) => {}
                    (TerminalOutcome::Rejected, Some(AgentHarnessError::Contract(_))) => {}
                    (_, other) => panic!("original execution outcome was lost: {other:?}"),
                }
            }
            other => panic!("expected explicit persistence failure, got {other}"),
        }
        let history = only_history(&conn);
        let cancelled = matches!(expected, TerminalOutcome::Cancelled);
        assert_eq!(
            history.run.status,
            if cancelled {
                AgentRunStatus::Cancelled
            } else {
                AgentRunStatus::Failed
            }
        );
        assert!(history.run.completed_at_ms.is_some());
        let message = history.run.error.as_deref().unwrap();
        assert!(message.contains("terminal history denied"));
        assert!(history.results.is_empty());
        assert_eq!(
            executions,
            u32::from(!matches!(expected, TerminalOutcome::Rejected))
        );
    }
}

fn workflow_with_tool(
    tool_name: AgentToolName,
    kind: AgentToolKind,
    max_tool_calls: u32,
) -> AgentWorkflowDefinition {
    AgentWorkflowDefinition {
        id: AgentWorkflowId::new("workflow.premise.graph").unwrap(),
        label: "Premise graph".to_string(),
        intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
        manifest: AgentToolManifest {
            tools: vec![AgentToolDefinition {
                name: tool_name,
                kind,
                description: "Mock tool".to_string(),
            }],
        },
        budget: AgentToolBudget {
            max_tool_calls,
            ..AgentToolBudget::default()
        },
        policy: AgentWorkflowPolicy::default(),
    }
}

struct MockProvider {
    requests: Vec<AgentToolRequest>,
}

impl AgentWorkflowProvider for MockProvider {
    fn next_tool_request(
        &mut self,
        _turn: AgentProviderTurn<'_>,
    ) -> Result<Option<AgentToolRequest>, AgentHarnessError> {
        if self.requests.is_empty() {
            return Ok(None);
        }
        Ok(Some(self.requests.remove(0)))
    }
}

struct MockTools;

impl AgentWorkflowToolExecutor for MockTools {
    fn execute_tool(
        &mut self,
        _request: &AgentToolRequest,
    ) -> Result<AgentToolResultPayload, AgentHarnessError> {
        Ok(AgentToolResultPayload::Text {
            text: "mock tool result".to_string(),
        })
    }
}
