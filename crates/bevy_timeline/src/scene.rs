use bevy::prelude::{Component, Entity, Resource, With, World};
use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};
use eidetic_core::timeline::track::TrackId;

#[derive(Component)]
pub struct TimelineSceneEntity;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TimelineTrackEntity {
    pub track_id: TrackId,
    pub level: StoryLevel,
    pub label: String,
    pub sort_order: u32,
    pub collapsed: bool,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TimelineClipEntity {
    pub node_id: NodeId,
    pub parent_id: Option<NodeId>,
    pub track_id: TrackId,
    pub level: StoryLevel,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub sort_order: u32,
    pub locked: bool,
    pub content_status: ContentStatus,
    pub arc_ids: Vec<ArcId>,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TimelineRelationshipEntity {
    pub relationship_id: RelationshipId,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    pub relationship_type: RelationshipType,
}

#[derive(Resource, Default)]
pub struct TimelineSceneStats {
    pub track_count: usize,
    pub clip_count: usize,
    pub relationship_count: usize,
}

pub fn rebuild_timeline_scene(world: &mut World, projection: &TimelineRenderProjection) {
    despawn_existing_scene(world);

    for track in &projection.tracks {
        world.spawn((
            TimelineSceneEntity,
            TimelineTrackEntity {
                track_id: track.track_id,
                level: track.level,
                label: track.label.clone(),
                sort_order: track.sort_order,
                collapsed: track.collapsed,
            },
        ));
    }

    for clip in &projection.clips {
        world.spawn((
            TimelineSceneEntity,
            TimelineClipEntity {
                node_id: clip.node_id,
                parent_id: clip.parent_id,
                track_id: clip.track_id,
                level: clip.level,
                name: clip.name.clone(),
                start_ms: clip.start_ms,
                end_ms: clip.end_ms,
                sort_order: clip.sort_order,
                locked: clip.locked,
                content_status: clip.content_status,
                arc_ids: clip.arc_ids.clone(),
            },
        ));
    }

    for relationship in &projection.relationships {
        world.spawn((
            TimelineSceneEntity,
            TimelineRelationshipEntity {
                relationship_id: relationship.relationship_id,
                from_node_id: relationship.from_node_id,
                to_node_id: relationship.to_node_id,
                relationship_type: relationship.relationship_type.clone(),
            },
        ));
    }

    world.resource_mut::<TimelineSceneStats>().track_count = projection.tracks.len();
    world.resource_mut::<TimelineSceneStats>().clip_count = projection.clips.len();
    world
        .resource_mut::<TimelineSceneStats>()
        .relationship_count = projection.relationships.len();
}

fn despawn_existing_scene(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<TimelineSceneEntity>>()
        .iter(world)
        .collect();

    for entity in entities {
        let _ = world.despawn(entity);
    }
}
