use eidetic_core::contracts::{AgentToolRequest, AgentWorkflowDefinition};
use serde::{Deserialize, Serialize};

use crate::agent_workflow_harness::{AgentHarnessError, AgentProviderTurn, AgentWorkflowProvider};

const DEFAULT_MAX_PROVIDER_RESPONSE_BYTES: usize = 16 * 1024;

pub trait AgentStructuredTextProvider {
    fn generate_structured_tool_turn(
        &mut self,
        prompt: StructuredToolPrompt<'_>,
    ) -> Result<String, AgentHarnessError>;
}

#[derive(Debug, Clone, Copy)]
pub struct StructuredToolLoopConfig {
    pub max_provider_response_bytes: usize,
}

impl Default for StructuredToolLoopConfig {
    fn default() -> Self {
        Self {
            max_provider_response_bytes: DEFAULT_MAX_PROVIDER_RESPONSE_BYTES,
        }
    }
}

pub struct StructuredToolLoopProvider<T> {
    provider: T,
    config: StructuredToolLoopConfig,
}

impl<T> StructuredToolLoopProvider<T> {
    pub fn new(provider: T) -> Self {
        Self {
            provider,
            config: StructuredToolLoopConfig::default(),
        }
    }

    pub fn with_config(provider: T, config: StructuredToolLoopConfig) -> Self {
        Self { provider, config }
    }

    pub fn into_inner(self) -> T {
        self.provider
    }
}

impl<T> AgentWorkflowProvider for StructuredToolLoopProvider<T>
where
    T: AgentStructuredTextProvider,
{
    fn next_tool_request(
        &mut self,
        turn: AgentProviderTurn<'_>,
    ) -> Result<Option<AgentToolRequest>, AgentHarnessError> {
        if self.config.max_provider_response_bytes == 0 {
            return Err(AgentHarnessError::Provider(
                "structured provider response byte budget must be greater than zero".to_string(),
            ));
        }

        let prompt_text = render_structured_tool_prompt(&turn)?;
        let response_text = self
            .provider
            .generate_structured_tool_turn(StructuredToolPrompt {
                workflow: turn.workflow,
                prompt_text: &prompt_text,
            })?;
        if response_text.len() > self.config.max_provider_response_bytes {
            return Err(AgentHarnessError::Provider(format!(
                "structured provider response exceeded {} bytes",
                self.config.max_provider_response_bytes
            )));
        }

        match parse_structured_tool_response(&response_text)? {
            StructuredToolResponse::Complete { .. } => Ok(None),
            StructuredToolResponse::ToolCall { request } => Ok(Some(request)),
        }
    }
}

pub struct StructuredToolPrompt<'a> {
    pub workflow: &'a AgentWorkflowDefinition,
    pub prompt_text: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum StructuredToolResponse {
    ToolCall { request: AgentToolRequest },
    Complete { summary: String },
}

fn render_structured_tool_prompt(
    turn: &AgentProviderTurn<'_>,
) -> Result<String, AgentHarnessError> {
    let workflow = serde_json::to_string(&turn.workflow)
        .map_err(|error| AgentHarnessError::Provider(error.to_string()))?;
    let completed_calls = serde_json::to_string(&turn.completed_calls)
        .map_err(|error| AgentHarnessError::Provider(error.to_string()))?;
    let completed_results = serde_json::to_string(&turn.completed_results)
        .map_err(|error| AgentHarnessError::Provider(error.to_string()))?;

    Ok(format!(
        concat!(
            "You are executing an Eidetic backend workflow.\n",
            "Return one strict JSON object only.\n",
            "To call a tool, return: ",
            r#"{{"status":"tool_call","request":{{"tool_name":"...","arguments":{{...}}}}}}"#,
            "\n",
            "To finish, return: ",
            r#"{{"status":"complete","summary":"..."}}"#,
            "\n",
            "Workflow JSON:\n{workflow}\n",
            "Completed tool calls JSON:\n{completed_calls}\n",
            "Completed tool results JSON:\n{completed_results}\n"
        ),
        workflow = workflow,
        completed_calls = completed_calls,
        completed_results = completed_results
    ))
}

fn parse_structured_tool_response(
    response_text: &str,
) -> Result<StructuredToolResponse, AgentHarnessError> {
    let trimmed = response_text.trim();
    if trimmed.is_empty() {
        return Err(AgentHarnessError::Provider(
            "structured provider returned an empty response".to_string(),
        ));
    }
    serde_json::from_str(trimmed)
        .map_err(|error| AgentHarnessError::Provider(format!("invalid structured JSON: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        AgentRun, AgentRunId, AgentRunStatus, AgentToolArguments, AgentToolBudget,
        AgentToolDefinition, AgentToolKind, AgentToolManifest, AgentToolName, AgentWorkflowId,
        AgentWorkflowIntent, AgentWorkflowPolicy, BibleGraphNodeId,
    };

    #[test]
    fn structured_provider_returns_tool_request_from_json_response() {
        let request = AgentToolRequest {
            tool_name: AgentToolName::new("read_bible_node").unwrap(),
            arguments: AgentToolArguments::ReadBibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            },
        };
        let response = serde_json::json!({
            "status": "tool_call",
            "request": request,
        })
        .to_string();
        let mut provider = StructuredToolLoopProvider::new(StaticTextProvider {
            response,
            prompts: Vec::new(),
        });

        let next_request = provider.next_tool_request(turn()).unwrap().unwrap();

        assert_eq!(next_request.tool_name.as_str(), "read_bible_node");
        let inner = provider.into_inner();
        assert_eq!(inner.prompts.len(), 1);
        assert!(inner.prompts[0].contains("Workflow JSON"));
    }

    #[test]
    fn structured_provider_returns_none_when_model_completes() {
        let mut provider = StructuredToolLoopProvider::new(StaticTextProvider {
            response: r#"{"status":"complete","summary":"done"}"#.to_string(),
            prompts: Vec::new(),
        });

        assert!(provider.next_tool_request(turn()).unwrap().is_none());
    }

    #[test]
    fn structured_provider_rejects_markdown_wrapped_json() {
        let mut provider = StructuredToolLoopProvider::new(StaticTextProvider {
            response: "```json\n{\"status\":\"complete\",\"summary\":\"done\"}\n```".to_string(),
            prompts: Vec::new(),
        });

        let error = provider.next_tool_request(turn()).unwrap_err();

        assert!(matches!(error, AgentHarnessError::Provider(_)));
    }

    #[test]
    fn structured_provider_enforces_response_byte_budget() {
        let mut provider = StructuredToolLoopProvider::with_config(
            StaticTextProvider {
                response: r#"{"status":"complete","summary":"done"}"#.to_string(),
                prompts: Vec::new(),
            },
            StructuredToolLoopConfig {
                max_provider_response_bytes: 8,
            },
        );

        let error = provider.next_tool_request(turn()).unwrap_err();

        assert!(error.to_string().contains("exceeded"));
    }

    fn turn<'a>() -> AgentProviderTurn<'a> {
        AgentProviderTurn {
            workflow: Box::leak(Box::new(workflow())),
            run: Box::leak(Box::new(AgentRun {
                id: AgentRunId::new(),
                workflow_id: AgentWorkflowId::new("workflow.premise.graph").unwrap(),
                status: AgentRunStatus::Running,
                intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
                created_at_ms: 10,
                completed_at_ms: None,
                error: None,
            })),
            completed_calls: &[],
            completed_results: &[],
        }
    }

    fn workflow() -> AgentWorkflowDefinition {
        AgentWorkflowDefinition {
            id: AgentWorkflowId::new("workflow.premise.graph").unwrap(),
            label: "Premise graph".to_string(),
            intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
            manifest: AgentToolManifest {
                tools: vec![AgentToolDefinition {
                    name: AgentToolName::new("read_bible_node").unwrap(),
                    kind: AgentToolKind::GraphRead,
                    description: "Read one bible node".to_string(),
                }],
            },
            budget: AgentToolBudget::default(),
            policy: AgentWorkflowPolicy::default(),
        }
    }

    struct StaticTextProvider {
        response: String,
        prompts: Vec<String>,
    }

    impl AgentStructuredTextProvider for StaticTextProvider {
        fn generate_structured_tool_turn(
            &mut self,
            prompt: StructuredToolPrompt<'_>,
        ) -> Result<String, AgentHarnessError> {
            self.prompts.push(prompt.prompt_text.to_string());
            Ok(self.response.clone())
        }
    }
}
