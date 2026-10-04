use eidetic_core::contracts::{
    CommandEnvelope, ProjectionEnvelope, PropagationProposalListProjection,
    RequestScriptImpactProposalCommand,
};
use serde::Serialize;

use crate::backend_error::BackendError;
use crate::history_store::{self, RecordChangeOutcome};
use crate::propagation_proposal_store;
use crate::script_impact_review;
use crate::state::{AppState, ServerEvent};

#[derive(Debug, Serialize)]
pub struct ScriptImpactProposalCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<PropagationProposalListProjection>,
}

pub async fn request_script_impact_proposal(
    state: &AppState,
    command: CommandEnvelope<RequestScriptImpactProposalCommand>,
) -> Result<ScriptImpactProposalCommandResponse, BackendError> {
    let path = crate::command_service_support::active_project_path(state)?;
    let capture_path = path.clone();
    let capture_command = command.clone();
    let prepared = tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&capture_path).map_err(internal)?;
        propagation_proposal_store::create_schema(&conn).map_err(internal)?;
        if let Some(outcome) = history_store::check_recorded_command(
            &conn,
            &capture_command,
            script_impact_review::REQUEST_TYPE,
        )
        .map_err(invalid)?
        {
            let projection =
                propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
                    .map_err(internal)?;
            return Ok::<_, BackendError>(Err(ScriptImpactProposalCommandResponse {
                outcome,
                projection,
            }));
        }
        let binding =
            script_impact_review::capture(&conn, &capture_command.payload).map_err(invalid)?;
        Ok(Ok(binding))
    })
    .await
    .map_err(internal)??;
    let binding = match prepared {
        Ok(binding) => binding,
        Err(replay) => return Ok(replay),
    };
    let config = state.ai_config.lock().clone();
    let text =
        crate::script_impact_prompt::preview_with_provider(&binding, move |prompt| async move {
            crate::ai_backends::Backend::from_config(&config)
                .generate(&prompt, &config)
                .await
        })
        .await
        .map_err(invalid)?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path).map_err(internal)?;
        let created_at_ms = u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(internal)?
                .as_millis(),
        )
        .map_err(internal)?;
        let outcome = script_impact_review::record_proposal(
            &mut conn,
            &command,
            binding,
            text,
            created_at_ms,
        )
        .map_err(invalid)?;
        let projection =
            propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
                .map_err(internal)?;
        Ok::<_, BackendError>(ScriptImpactProposalCommandResponse {
            outcome,
            projection,
        })
    })
    .await
    .map_err(internal)??;
    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
    }
    Ok(response)
}

fn invalid(error: impl std::fmt::Display) -> BackendError {
    BackendError::bad_request(error.to_string())
}
fn internal(error: impl std::fmt::Display) -> BackendError {
    BackendError::internal(error.to_string())
}

#[cfg(test)]
#[path = "script_impact_review_service_tests.rs"]
mod tests;
