use bevy::prelude::{App, Resource};
use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::NodeId;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineRendererCommand {
    SelectNode { node_id: NodeId },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TimelineRendererError {
    #[error("timeline projection has not been loaded")]
    MissingProjection,
    #[error("timeline projection does not contain node {node_id:?}")]
    UnknownNode { node_id: NodeId },
}

#[derive(Resource, Default)]
struct TimelineRenderState {
    projection: Option<TimelineRenderProjection>,
}

#[derive(Resource, Default)]
struct TimelineRendererCommandQueue {
    commands: Vec<TimelineRendererCommand>,
}

pub struct TimelineRendererApp {
    app: App,
}

impl Default for TimelineRendererApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineRendererApp {
    pub fn new() -> Self {
        let mut app = App::new();
        app.insert_resource(TimelineRenderState::default());
        app.insert_resource(TimelineRendererCommandQueue::default());
        Self { app }
    }

    pub fn set_projection(&mut self, projection: TimelineRenderProjection) {
        self.app
            .world_mut()
            .resource_mut::<TimelineRenderState>()
            .projection = Some(projection);
    }

    pub fn projection_clip_count(&self) -> usize {
        self.app
            .world()
            .resource::<TimelineRenderState>()
            .projection
            .as_ref()
            .map(|projection| projection.clips.len())
            .unwrap_or_default()
    }

    pub fn select_node(&mut self, node_id: NodeId) -> Result<(), TimelineRendererError> {
        let has_node = {
            let state = self.app.world().resource::<TimelineRenderState>();
            let projection = state
                .projection
                .as_ref()
                .ok_or(TimelineRendererError::MissingProjection)?;
            projection.clips.iter().any(|clip| clip.node_id == node_id)
        };

        if !has_node {
            return Err(TimelineRendererError::UnknownNode { node_id });
        }

        self.app
            .world_mut()
            .resource_mut::<TimelineRendererCommandQueue>()
            .commands
            .push(TimelineRendererCommand::SelectNode { node_id });
        Ok(())
    }

    pub fn drain_commands(&mut self) -> Vec<TimelineRendererCommand> {
        std::mem::take(
            &mut self
                .app
                .world_mut()
                .resource_mut::<TimelineRendererCommandQueue>()
                .commands,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{TimelineRenderClip, TimelineRenderTrack};
    use eidetic_core::timeline::node::{ContentStatus, StoryLevel};
    use eidetic_core::timeline::track::TrackId;

    #[test]
    fn renderer_app_receives_projection_and_emits_validated_selection_command() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();

        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(renderer.projection_clip_count(), 1);
        assert_eq!(renderer.select_node(node_id), Ok(()));
        assert_eq!(
            renderer.drain_commands(),
            vec![TimelineRendererCommand::SelectNode { node_id }]
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_rejects_selection_before_projection_load() {
        let mut renderer = TimelineRendererApp::new();
        let node_id = NodeId::new();

        assert_eq!(
            renderer.select_node(node_id),
            Err(TimelineRendererError::MissingProjection)
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_rejects_unknown_node_selection() {
        let mut renderer = TimelineRendererApp::new();
        let known_node_id = NodeId::new();
        let unknown_node_id = NodeId::new();
        renderer.set_projection(projection_with_node(known_node_id));

        assert_eq!(
            renderer.select_node(unknown_node_id),
            Err(TimelineRendererError::UnknownNode {
                node_id: unknown_node_id
            })
        );
        assert!(renderer.drain_commands().is_empty());
    }

    fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
        let track_id = TrackId::new();
        TimelineRenderProjection {
            total_duration_ms: 10_000,
            tracks: vec![TimelineRenderTrack {
                track_id,
                level: StoryLevel::Scene,
                label: "Scenes".to_string(),
                sort_order: 30,
                collapsed: false,
            }],
            clips: vec![TimelineRenderClip {
                node_id,
                parent_id: None,
                track_id,
                level: StoryLevel::Scene,
                name: "Beach argument".to_string(),
                start_ms: 1_000,
                end_ms: 4_000,
                sort_order: 10,
                locked: false,
                content_status: ContentStatus::NotesOnly,
                beat_type: None,
                arc_ids: Vec::new(),
            }],
            relationships: Vec::new(),
        }
    }
}
