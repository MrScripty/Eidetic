use bevy::prelude::{Component, Entity, Resource, With, World};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNodeId, BibleGraphSchemaKey,
    BibleRenderGraphPosition, BibleRenderGraphProjection, ContextInfluenceId, ContextInfluenceKind,
    ContextInfluenceProvenance,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel};

#[derive(Component)]
pub struct BibleGraphSceneEntity;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNodeEntity {
    pub node_id: BibleGraphNodeId,
    pub parent_id: Option<BibleGraphNodeId>,
    pub schema_key: BibleGraphSchemaKey,
    pub label: String,
    pub system_owned: bool,
    pub sort_order: u32,
    pub depth: u32,
    pub position: BibleRenderGraphPosition,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct BibleGraphEdgeEntity {
    pub edge_id: BibleGraphEdgeId,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub edge_kind: BibleGraphEdgeKind,
    pub label: String,
    pub directed: bool,
    pub sort_order: u32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphInfluenceEntity {
    pub influence_id: ContextInfluenceId,
    pub timeline_node_id: NodeId,
    pub source_layer: StoryLevel,
    pub influence_kind: ContextInfluenceKind,
    pub confidence: f32,
    pub reason: String,
    pub provenance: ContextInfluenceProvenance,
    pub bible_node_id: Option<BibleGraphNodeId>,
    pub bible_edge_id: Option<BibleGraphEdgeId>,
    pub sort_order: u32,
}

#[derive(Resource, Default)]
pub struct BibleGraphSceneStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub influence_count: usize,
}

pub fn rebuild_bible_graph_scene(world: &mut World, projection: &BibleRenderGraphProjection) {
    despawn_existing_scene(world);

    for node in &projection.nodes {
        world.spawn((
            BibleGraphSceneEntity,
            BibleGraphNodeEntity {
                node_id: node.node_id.clone(),
                parent_id: node.parent_id.clone(),
                schema_key: node.schema_key.clone(),
                label: node.label.clone(),
                system_owned: node.system_owned,
                sort_order: node.sort_order,
                depth: node.depth,
                position: node.position,
            },
        ));
    }

    for edge in &projection.edges {
        world.spawn((
            BibleGraphSceneEntity,
            BibleGraphEdgeEntity {
                edge_id: edge.edge_id.clone(),
                from_node_id: edge.from_node_id.clone(),
                to_node_id: edge.to_node_id.clone(),
                edge_kind: edge.edge_kind.clone(),
                label: edge.label.clone(),
                directed: edge.directed,
                sort_order: edge.sort_order,
            },
        ));
    }

    for influence in &projection.influences {
        world.spawn((
            BibleGraphSceneEntity,
            BibleGraphInfluenceEntity {
                influence_id: influence.influence_id,
                timeline_node_id: influence.timeline_node_id,
                source_layer: influence.source_layer,
                influence_kind: influence.influence_kind.clone(),
                confidence: influence.confidence,
                reason: influence.reason.clone(),
                provenance: influence.provenance.clone(),
                bible_node_id: influence.bible_node_id.clone(),
                bible_edge_id: influence.bible_edge_id.clone(),
                sort_order: influence.sort_order,
            },
        ));
    }

    world.resource_mut::<BibleGraphSceneStats>().node_count = projection.nodes.len();
    world.resource_mut::<BibleGraphSceneStats>().edge_count = projection.edges.len();
    world.resource_mut::<BibleGraphSceneStats>().influence_count = projection.influences.len();
}

fn despawn_existing_scene(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<BibleGraphSceneEntity>>()
        .iter(world)
        .collect();

    for entity in entities {
        let _ = world.despawn(entity);
    }
}
