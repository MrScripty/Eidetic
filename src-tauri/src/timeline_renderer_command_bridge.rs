use eidetic_bevy_timeline::TimelineRendererCommand;
use eidetic_core::contracts::{
    CommandEnvelope, CreateTimelineChildFromParentCommand, CreateTimelineRelationshipCommand,
    DeleteTimelineNodeCommand, SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
};
use eidetic_server::state::AppState;
use eidetic_server::{command_service, projection_service};
use std::time::Duration;
use tauri::Manager;
use tokio::sync::watch;

use crate::bevy_timeline_host::DesktopTimelineRendererOwner;

pub(crate) fn spawn_timeline_renderer_command_bridge(
    app: tauri::AppHandle,
    state: AppState,
    mut shutdown: watch::Receiver<bool>,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = shutdown.changed() => break,
                _ = interval.tick() => {}
            }
            let Some(owner) = app.try_state::<DesktopTimelineRendererOwner>() else {
                continue;
            };
            let commands = match owner.drain_commands() {
                Ok(commands) => commands,
                Err(error) => {
                    tracing::warn!("failed to drain timeline renderer commands: {error:?}");
                    continue;
                }
            };

            for command in commands {
                if let Err(error) = handle_timeline_renderer_command(&state, command.clone()).await
                {
                    tracing::warn!(
                        "failed to apply timeline renderer command {command:?}: {error:?}"
                    );
                }
            }
        }
    })
}

async fn handle_timeline_renderer_command(
    state: &AppState,
    command: TimelineRendererCommand,
) -> Result<(), String> {
    if let TimelineRendererCommand::SelectNode { node_id } = command {
        state.select_timeline_node(Some(node_id));
        return Ok(());
    }
    if let TimelineRendererCommand::SetPlayhead { position_ms } = command {
        set_timeline_playhead_from_renderer(state, position_ms).await?;
        return Ok(());
    }

    apply_timeline_renderer_command(state, command).await
}

async fn set_timeline_playhead_from_renderer(
    state: &AppState,
    position_ms: u64,
) -> Result<(), String> {
    let projection = projection_service::timeline_render_projection(state)
        .await
        .map_err(|error| error.to_string())?;
    let position_ms =
        clamp_timeline_renderer_playhead(position_ms, projection.payload.total_duration_ms);
    state.set_timeline_playhead(position_ms);
    Ok(())
}

fn clamp_timeline_renderer_playhead(position_ms: u64, total_duration_ms: u64) -> u64 {
    position_ms.min(total_duration_ms)
}

async fn apply_timeline_renderer_command(
    state: &AppState,
    command: TimelineRendererCommand,
) -> Result<(), String> {
    match timeline_renderer_mutation_command(command) {
        Some(TimelineRendererMutationCommand::SetNodeRange(command)) => {
            command_service::set_timeline_node_range(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        Some(TimelineRendererMutationCommand::DeleteNode(command)) => {
            command_service::delete_timeline_node(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        Some(TimelineRendererMutationCommand::CreateChildFromParent(command)) => {
            command_service::create_timeline_child_from_parent_core_command(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        Some(TimelineRendererMutationCommand::CreateRelationship(command)) => {
            command_service::create_timeline_relationship_from_core_command(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        Some(TimelineRendererMutationCommand::SplitNode(command)) => {
            command_service::split_timeline_node_from_core_command(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        None => Ok(()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TimelineRendererMutationCommand {
    SetNodeRange(CommandEnvelope<SetTimelineNodeRangeCommand>),
    DeleteNode(CommandEnvelope<DeleteTimelineNodeCommand>),
    CreateChildFromParent(CommandEnvelope<CreateTimelineChildFromParentCommand>),
    CreateRelationship(CommandEnvelope<CreateTimelineRelationshipCommand>),
    SplitNode(CommandEnvelope<SplitTimelineNodeCommand>),
}

fn timeline_renderer_mutation_command(
    command: TimelineRendererCommand,
) -> Option<TimelineRendererMutationCommand> {
    match command {
        TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms,
            end_ms,
        } => Some(TimelineRendererMutationCommand::SetNodeRange(
            CommandEnvelope::new(SetTimelineNodeRangeCommand {
                node_id,
                start_ms,
                end_ms,
            }),
        )),
        TimelineRendererCommand::DeleteNode { node_id } => {
            Some(TimelineRendererMutationCommand::DeleteNode(
                CommandEnvelope::new(DeleteTimelineNodeCommand { node_id }),
            ))
        }
        TimelineRendererCommand::CreateChildFromParent { node_id, parent_id } => {
            Some(TimelineRendererMutationCommand::CreateChildFromParent(
                CommandEnvelope::new(CreateTimelineChildFromParentCommand { node_id, parent_id }),
            ))
        }
        TimelineRendererCommand::CreateRelationship {
            relationship_id,
            from_node_id,
            to_node_id,
            relationship_type,
        } => Some(TimelineRendererMutationCommand::CreateRelationship(
            CommandEnvelope::new(CreateTimelineRelationshipCommand {
                relationship_id,
                from_node_id,
                to_node_id,
                relationship_type,
            }),
        )),
        TimelineRendererCommand::SplitNode {
            node_id,
            at_ms,
            left_node_id,
            right_node_id,
        } => Some(TimelineRendererMutationCommand::SplitNode(
            CommandEnvelope::new(SplitTimelineNodeCommand {
                node_id,
                at_ms,
                left_node_id,
                right_node_id,
            }),
        )),
        TimelineRendererCommand::SelectNode { .. }
        | TimelineRendererCommand::SetPlayhead { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TimelineRendererMutationCommand, clamp_timeline_renderer_playhead,
        timeline_renderer_mutation_command,
    };
    use eidetic_bevy_timeline::TimelineRendererCommand;
    use eidetic_core::contracts::DeleteTimelineNodeCommand;
    use eidetic_core::timeline::node::NodeId;
    use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};

    #[test]
    fn maps_timeline_renderer_range_commands_to_backend_commands() {
        let node_id = NodeId::new();

        let Some(TimelineRendererMutationCommand::SetNodeRange(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::SetNodeRange {
                node_id,
                start_ms: 1_000,
                end_ms: 2_000,
            })
        else {
            panic!("expected set range mutation command");
        };

        assert_eq!(command.payload.node_id, node_id);
        assert_eq!(command.payload.start_ms, 1_000);
        assert_eq!(command.payload.end_ms, 2_000);
    }

    #[test]
    fn maps_timeline_renderer_delete_commands_to_backend_commands() {
        let node_id = NodeId::new();

        assert!(matches!(
            timeline_renderer_mutation_command(TimelineRendererCommand::DeleteNode { node_id }),
            Some(TimelineRendererMutationCommand::DeleteNode(command))
                if command.payload == DeleteTimelineNodeCommand { node_id }
        ));
    }

    #[test]
    fn maps_timeline_renderer_create_child_intents_to_backend_commands() {
        let node_id = NodeId::new();
        let parent_id = NodeId::new();

        let Some(TimelineRendererMutationCommand::CreateChildFromParent(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::CreateChildFromParent {
                node_id,
                parent_id,
            })
        else {
            panic!("expected create child mutation command");
        };

        assert_eq!(command.payload.node_id, node_id);
        assert_eq!(command.payload.parent_id, parent_id);
    }

    #[test]
    fn maps_timeline_renderer_create_relationship_commands_to_backend_commands() {
        let relationship_id = RelationshipId::new();
        let from_node_id = NodeId::new();
        let to_node_id = NodeId::new();

        let Some(TimelineRendererMutationCommand::CreateRelationship(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::CreateRelationship {
                relationship_id,
                from_node_id,
                to_node_id,
                relationship_type: RelationshipType::Causal,
            })
        else {
            panic!("expected create relationship mutation command");
        };

        assert_eq!(command.payload.relationship_id, relationship_id);
        assert_eq!(command.payload.from_node_id, from_node_id);
        assert_eq!(command.payload.to_node_id, to_node_id);
        assert_eq!(command.payload.relationship_type, RelationshipType::Causal);
    }

    #[test]
    fn maps_timeline_renderer_split_commands_to_backend_commands() {
        let node_id = NodeId::new();
        let left_node_id = NodeId::new();
        let right_node_id = NodeId::new();

        let Some(TimelineRendererMutationCommand::SplitNode(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::SplitNode {
                node_id,
                at_ms: 4_000,
                left_node_id,
                right_node_id,
            })
        else {
            panic!("expected split mutation command");
        };

        assert_eq!(command.payload.node_id, node_id);
        assert_eq!(command.payload.at_ms, 4_000);
        assert_eq!(command.payload.left_node_id, left_node_id);
        assert_eq!(command.payload.right_node_id, right_node_id);
    }

    #[test]
    fn ignores_timeline_renderer_commands_without_backend_mutation() {
        let node_id = NodeId::new();

        assert_eq!(
            timeline_renderer_mutation_command(TimelineRendererCommand::SelectNode { node_id }),
            None
        );
        assert_eq!(
            timeline_renderer_mutation_command(TimelineRendererCommand::SetPlayhead {
                position_ms: 42_000,
            }),
            None
        );
    }

    #[test]
    fn clamps_renderer_playhead_commands_to_backend_projection_duration() {
        assert_eq!(clamp_timeline_renderer_playhead(42_500, 120_000), 42_500);
        assert_eq!(clamp_timeline_renderer_playhead(240_000, 120_000), 120_000);
    }
}
