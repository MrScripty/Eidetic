//! Explicit provider I/O with canonical capture, exact replay and atomic proposal storage.
use crate::state::{AppState, ServerEvent};
use crate::{
    backend_error::BackendError, history_store, propagation_proposal_store, script_fact_evidence,
    script_fact_proposal,
};
use eidetic_core::contracts::*;

#[derive(serde::Serialize)]
pub struct ScriptFactProposalCommandResponse {
    pub(crate) outcome: history_store::RecordChangeOutcome,
    pub(crate) projection: ProjectionEnvelope<PropagationProposalListProjection>,
}

pub async fn request_script_fact_proposal(
    state: &AppState,
    command: CommandEnvelope<RequestScriptFactProposalCommand>,
) -> Result<ScriptFactProposalCommandResponse, BackendError> {
    let path = crate::command_service_support::active_project_path(state)?;
    let capture_path = path.clone();
    let capture_command = command.clone();
    let prepared = tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&capture_path).map_err(internal)?;
        propagation_proposal_store::create_schema(&conn).map_err(internal)?;
        if let Some(outcome) = history_store::check_recorded_command(
            &conn,
            &capture_command,
            script_fact_proposal::REQUEST_TYPE,
        )
        .map_err(invalid)?
        {
            return Ok::<_, BackendError>(Err(ScriptFactProposalCommandResponse {
                outcome,
                projection: propagation_proposal_store::load_propagation_proposal_list_projection(
                    &conn,
                )
                .map_err(internal)?,
            }));
        }
        let binding =
            script_fact_evidence::capture(&conn, &capture_command.payload).map_err(invalid)?;
        Ok(Ok(binding))
    })
    .await
    .map_err(internal)??;
    let binding = match prepared {
        Ok(binding) => binding,
        Err(replay) => return Ok(replay),
    };
    let config = state.ai_config.lock().clone();
    let suggestion =
        script_fact_proposal::analyze_with_provider(&binding, move |prompt| async move {
            crate::ai_backends::Backend::from_config(&config)
                .generate(&prompt, &config)
                .await
        })
        .await
        .map_err(invalid)?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path).map_err(internal)?;
        let at = u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(internal)?
                .as_millis(),
        )
        .map_err(internal)?;
        let outcome = script_fact_proposal::record(&mut conn, &command, binding, suggestion, at)
            .map_err(invalid)?;
        Ok::<_, BackendError>(ScriptFactProposalCommandResponse {
            outcome,
            projection: propagation_proposal_store::load_propagation_proposal_list_projection(
                &conn,
            )
            .map_err(internal)?,
        })
    })
    .await
    .map_err(internal)??;
    if response.outcome == history_store::RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
    }
    Ok(response)
}
fn invalid(e: impl std::fmt::Display) -> BackendError {
    BackendError::bad_request(e.to_string())
}
fn internal(e: impl std::fmt::Display) -> BackendError {
    BackendError::internal(e.to_string())
}

#[cfg(test)]
#[path = "script_fact_proposal_service_tests.rs"]
mod tests;
