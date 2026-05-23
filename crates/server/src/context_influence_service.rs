use eidetic_core::contracts::{
    CommandEnvelope, ContextEvaluation, ContextInfluenceProjection,
    ContextInfluenceProjectionRequest, ContextStackProjection, ContextStackProjectionRequest,
    ObjectKind, ProjectionEnvelope, ProjectionVersion, RecordContextEvaluationCommand,
};

use crate::backend_error::BackendError;
use crate::command_service_support::{active_project_path, map_history_error};
use crate::context_influence_store;
use crate::history_store::{self, RecordChangeOutcome};
use crate::state::{AppState, ServerEvent};
use crate::timeline_node_store;

pub async fn record_context_evaluation(
    state: &AppState,
    command: CommandEnvelope<RecordContextEvaluationCommand>,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, BackendError> {
    let path = active_project_path(state)?;
    let (outcome, projection) =
        tokio::task::spawn_blocking(move || record_context_evaluation_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("context evaluation task failed: {error}"))
            })??;
    if outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::ContextInfluenceChanged {
            target_node_id: projection.payload.target_node_id.0,
        });
    }
    Ok(projection)
}

pub async fn context_influence_projection(
    state: &AppState,
    request: ContextInfluenceProjectionRequest,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_context_influence_projection_at_path(path, request))
        .await
        .map_err(|error| {
            BackendError::internal(format!("context influence projection task failed: {error}"))
        })?
}

pub async fn context_stack_projection(
    state: &AppState,
    request: ContextStackProjectionRequest,
) -> Result<ProjectionEnvelope<ContextStackProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_context_stack_projection_at_path(path, request))
        .await
        .map_err(|error| {
            BackendError::internal(format!("context stack projection task failed: {error}"))
        })?
}

fn record_context_evaluation_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<RecordContextEvaluationCommand>,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<ContextInfluenceProjection>,
    ),
    BackendError,
> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let created_at_ms = command.payload.evaluation.created_at_ms;
    let outcome =
        context_influence_store::record_context_evaluation(&mut conn, &command, created_at_ms)
            .map_err(map_history_error)?;
    let projection = context_influence_store::load_context_influence_projection(
        &conn,
        command.payload.evaluation.target_node_id,
    )
    .map_err(map_history_error)?;
    let projection = projection
        .ok_or_else(|| BackendError::internal("missing recorded context influence projection"))?;
    Ok((outcome, projection))
}

fn load_context_influence_projection_at_path(
    path: std::path::PathBuf,
    request: ContextInfluenceProjectionRequest,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    context_influence_store::load_context_influence_projection(&conn, request.target_node_id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::not_found("context influence projection not found"))
}

fn load_context_stack_projection_at_path(
    path: std::path::PathBuf,
    request: ContextStackProjectionRequest,
) -> Result<ProjectionEnvelope<ContextStackProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let nodes = timeline_node_store::load_node_ancestor_stack(&conn, request.target_node_id)
        .map_err(map_history_error)?;
    let mut projection = ContextStackProjection::from_nodes(&nodes, request.target_node_id)
        .ok_or_else(|| BackendError::not_found("context stack target node not found"))?;
    let context_node_ids: Vec<_> = projection
        .layers
        .iter()
        .map(|layer| layer.node_id)
        .collect();
    let evaluations =
        context_influence_store::load_latest_context_evaluations(&conn, &context_node_ids)
            .map_err(map_history_error)?;
    apply_distilled_context_overrides(&mut projection, &evaluations);
    let summary = history_store::load_revision_summary_for_kind(&conn, ObjectKind::TimelineNode)
        .map_err(map_history_error)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}

fn apply_distilled_context_overrides(
    projection: &mut ContextStackProjection,
    evaluations: &[ContextEvaluation],
) {
    for layer in &mut projection.layers {
        let Some(evaluation) = evaluations
            .iter()
            .find(|evaluation| evaluation.target_node_id == layer.node_id)
        else {
            continue;
        };
        if let Some(distilled_context) = &evaluation.distilled_context {
            layer.distilled_context = Some(distilled_context.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use eidetic_core::contracts::{
        BibleGraphNodeId, CommandEnvelope, ContextEvaluation, ContextEvaluationId,
        ContextEvaluationTaskKind, ContextInfluenceKind, ContextInfluenceProvenance,
        ContextInfluenceRecord, ContextLayerRole, ContextStackLayer,
        RecordContextEvaluationCommand,
    };
    use eidetic_core::timeline::node::{NodeId, StoryLevel};

    use super::*;

    #[test]
    fn context_stack_uses_recorded_distilled_parent_context() {
        let parent_id = NodeId::new();
        let child_id = NodeId::new();
        let mut projection = ContextStackProjection {
            target_node_id: child_id,
            layers: vec![
                ContextStackLayer {
                    node_id: parent_id,
                    level: StoryLevel::Act,
                    label: "Act One".to_string(),
                    role: ContextLayerRole::Inherited,
                    distilled_context: Some("Old timeline recap.".to_string()),
                    sort_order: 0,
                },
                ContextStackLayer {
                    node_id: child_id,
                    level: StoryLevel::Scene,
                    label: "Scene".to_string(),
                    role: ContextLayerRole::Target,
                    distilled_context: None,
                    sort_order: 0,
                },
            ],
        };
        let evaluations = vec![ContextEvaluation {
            id: ContextEvaluationId::new(),
            target_node_id: parent_id,
            task_kind: ContextEvaluationTaskKind::GenerateTimelineContext,
            summary: "Parent context".to_string(),
            distilled_context: Some("Refined parent context.".to_string()),
            created_at_ms: 200,
        }];

        apply_distilled_context_overrides(&mut projection, &evaluations);

        assert_eq!(
            projection.layers[0].distilled_context.as_deref(),
            Some("Refined parent context.")
        );
        assert!(projection.layers[1].distilled_context.is_none());
    }

    #[test]
    fn context_evaluation_history_uses_payload_timestamp() {
        let path = std::env::temp_dir().join(format!(
            "eidetic-context-evaluation-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let command = context_evaluation_command(42_000);

        record_context_evaluation_at_path(path.clone(), command.clone()).unwrap();

        let conn = rusqlite::Connection::open(&path).unwrap();
        let command_created_at_ms: i64 = conn
            .query_row(
                "SELECT created_at_ms FROM commands WHERE id = ?1",
                [command.id.0.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        let event_created_at_ms: i64 = conn
            .query_row(
                "SELECT created_at_ms FROM change_events WHERE command_id = ?1",
                [command.id.0.to_string()],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(command_created_at_ms, 42_000);
        assert_eq!(event_created_at_ms, 42_000);

        std::fs::remove_file(path).unwrap();
    }

    fn context_evaluation_command(
        created_at_ms: u64,
    ) -> CommandEnvelope<RecordContextEvaluationCommand> {
        let target_node_id = NodeId::new();
        let evaluation_id = ContextEvaluationId::new();
        CommandEnvelope::new(RecordContextEvaluationCommand {
            evaluation: ContextEvaluation {
                id: evaluation_id,
                target_node_id,
                task_kind: ContextEvaluationTaskKind::GenerateTimelineContext,
                summary: "Scene context evaluation".to_string(),
                distilled_context: Some("Harbor weather shapes the scene.".to_string()),
                created_at_ms,
            },
            influences: vec![ContextInfluenceRecord {
                id: eidetic_core::contracts::ContextInfluenceId::new(),
                evaluation_id,
                timeline_node_id: target_node_id,
                source_layer: StoryLevel::Scene,
                influence_kind: ContextInfluenceKind::Direct,
                confidence: 0.91,
                reason: "Scene takes place at the harbor.".to_string(),
                provenance: ContextInfluenceProvenance::AiSelected,
                bible_node_id: Some(BibleGraphNodeId::new("node.place.harbor").unwrap()),
                bible_edge_id: None,
                introduced_by_node_id: Some(target_node_id),
                sort_order: 1,
            }],
        })
    }
}
