use eidetic_bevy_timeline::TimelineRendererCommand;
use eidetic_core::contracts::{
    CommandEnvelope, CreateTimelineNodeCommand, DeleteTimelineNodeCommand,
    SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
};
use eidetic_server::command_service;
use eidetic_server::state::AppState;
use std::time::Duration;
use tauri::Manager;

use crate::bevy_timeline_host::DesktopTimelineRendererOwner;

pub(crate) fn spawn_timeline_renderer_command_bridge(
    app: tauri::AppHandle,
    state: AppState,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;
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

    apply_timeline_renderer_command(state, command).await
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
        Some(TimelineRendererMutationCommand::CreateNode(command)) => {
            command_service::create_timeline_node_from_core_command(state, command)
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
    CreateNode(CommandEnvelope<CreateTimelineNodeCommand>),
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
        TimelineRendererCommand::CreateNode {
            node_id,
            parent_id,
            level,
            name,
            start_ms,
            end_ms,
            beat_type,
        } => Some(TimelineRendererMutationCommand::CreateNode(
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id,
                parent_id,
                level,
                name,
                start_ms,
                end_ms,
                beat_type,
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
        TimelineRendererCommand::SelectNode { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{TimelineRendererMutationCommand, timeline_renderer_mutation_command};
    use eidetic_bevy_timeline::TimelineRendererCommand;
    use eidetic_core::contracts::DeleteTimelineNodeCommand;
    use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};

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
    fn maps_timeline_renderer_create_commands_to_backend_commands() {
        let node_id = NodeId::new();
        let parent_id = Some(NodeId::new());

        let Some(TimelineRendererMutationCommand::CreateNode(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::CreateNode {
                node_id,
                parent_id,
                level: StoryLevel::Beat,
                name: "Argument escalates".to_string(),
                start_ms: 3_000,
                end_ms: 5_000,
                beat_type: Some(BeatType::Escalation),
            })
        else {
            panic!("expected create mutation command");
        };

        assert_eq!(command.payload.node_id, node_id);
        assert_eq!(command.payload.parent_id, parent_id);
        assert_eq!(command.payload.level, StoryLevel::Beat);
        assert_eq!(command.payload.name, "Argument escalates");
        assert_eq!(command.payload.start_ms, 3_000);
        assert_eq!(command.payload.end_ms, 5_000);
        assert_eq!(command.payload.beat_type, Some(BeatType::Escalation));
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
    }
}
