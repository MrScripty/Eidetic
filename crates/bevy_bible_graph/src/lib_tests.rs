use super::*;
use eidetic_core::contracts::{
    BibleGraphEdgeKind, BibleGraphSchemaKey, BibleRenderGraphEdge, BibleRenderGraphInfluence,
    BibleRenderGraphNode, BibleRenderGraphPosition, ContextInfluenceKind,
    ContextInfluenceProvenance,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel};

#[test]
fn renderer_app_receives_projection_and_emits_validated_selection_command() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let mut renderer = BibleGraphRendererApp::new();

    renderer
        .set_projection(projection_with_node(node_id.clone()))
        .unwrap();

    assert_eq!(renderer.projection_node_count(), 1);
    assert_eq!(renderer.select_node(node_id.clone()), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![BibleGraphRendererCommand::SelectNode { node_id }]
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_uses_bounded_command_queue() {
    assert_eq!(BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, 128);

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let mut renderer = BibleGraphRendererApp::new();
    renderer
        .set_projection(projection_with_node(node_id.clone()))
        .unwrap();

    for _ in 0..BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY {
        assert_eq!(renderer.inspect_node(node_id.clone()), Ok(()));
    }

    assert_eq!(
        renderer.inspect_node(node_id),
        Err(BibleGraphRendererError::CommandQueueFull)
    );
    assert_eq!(
        renderer.drain_commands().len(),
        BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_rebuilds_scene_entities_from_projection() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let mut renderer = BibleGraphRendererApp::new();

    renderer
        .set_projection(projection_with_edge(node_id))
        .unwrap();
    assert_eq!(renderer.scene_counts(), (2, 1));
    assert_eq!(renderer.influence_count(), 0);

    renderer
        .set_projection(BibleRenderGraphProjection {
            focused_root_id: None,
            selected_node_id: None,
            selected_timeline_node_id: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            neighborhoods: Vec::new(),
            influences: Vec::new(),
        })
        .unwrap();
    assert_eq!(renderer.scene_counts(), (0, 0));
    assert_eq!(renderer.influence_count(), 0);
}

#[test]
fn renderer_app_rejects_projection_above_full_rebuild_envelope() {
    let mut renderer = BibleGraphRendererApp::new();
    let projection = projection_with_node_count(BIBLE_GRAPH_FULL_REBUILD_NODE_LIMIT + 1);

    assert_eq!(
        renderer.set_projection(projection),
        Err(
            BibleGraphRendererError::ProjectionExceedsPrototypeRebuildLimit {
                node_count: BIBLE_GRAPH_FULL_REBUILD_NODE_LIMIT + 1,
                edge_count: 0,
                node_limit: BIBLE_GRAPH_FULL_REBUILD_NODE_LIMIT,
                edge_limit: BIBLE_GRAPH_FULL_REBUILD_EDGE_LIMIT,
            }
        )
    );
    assert_eq!(renderer.scene_counts(), (0, 0));
}

#[test]
fn renderer_app_rejects_selection_before_projection_load() {
    let mut renderer = BibleGraphRendererApp::new();
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();

    assert_eq!(
        renderer.select_node(node_id),
        Err(BibleGraphRendererError::MissingProjection)
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_rejects_unknown_node_selection() {
    let mut renderer = BibleGraphRendererApp::new();
    let known_node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let unknown_node_id = BibleGraphNodeId::new("node.character.nope").unwrap();
    renderer
        .set_projection(projection_with_node(known_node_id))
        .unwrap();

    assert_eq!(
        renderer.inspect_node(unknown_node_id.clone()),
        Err(BibleGraphRendererError::UnknownNode {
            node_id: unknown_node_id
        })
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_returns_neighborhood_indexes_from_projection() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let mut renderer = BibleGraphRendererApp::new();
    renderer
        .set_projection(projection_with_edge(node_id.clone()))
        .unwrap();

    assert_eq!(
        renderer.edge_ids_for_node(&node_id),
        Ok(vec![edge_id.clone()])
    );
    assert_eq!(renderer.select_edge(edge_id.clone()), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![BibleGraphRendererCommand::SelectEdge { edge_id }]
    );
}

#[test]
fn renderer_app_rejects_unknown_edge_selection() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let unknown_edge_id = BibleGraphEdgeId::new("edge.unknown").unwrap();
    let mut renderer = BibleGraphRendererApp::new();
    renderer
        .set_projection(projection_with_node(node_id))
        .unwrap();

    assert_eq!(
        renderer.select_edge(unknown_edge_id.clone()),
        Err(BibleGraphRendererError::UnknownEdge {
            edge_id: unknown_edge_id
        })
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_indexes_influence_highlights_from_projection() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId::new();
    let mut renderer = BibleGraphRendererApp::new();

    renderer
        .set_projection(projection_with_influence(
            node_id.clone(),
            edge_id.clone(),
            influence_id,
        ))
        .unwrap();

    assert_eq!(renderer.scene_counts(), (2, 1));
    assert_eq!(renderer.influence_count(), 1);
    assert_eq!(
        renderer.influence_ids_for_node(&node_id),
        Ok(vec![influence_id])
    );
    assert_eq!(
        renderer.influence_ids_for_edge(&edge_id),
        Ok(vec![influence_id])
    );
    assert_eq!(renderer.select_influence(influence_id), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![BibleGraphRendererCommand::SelectInfluence { influence_id }]
    );
}

#[test]
fn renderer_app_exposes_projection_derived_visual_snapshot() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId::new();
    let mut renderer = BibleGraphRendererApp::new();

    renderer
        .set_projection(projection_with_influence(
            node_id.clone(),
            edge_id.clone(),
            influence_id,
        ))
        .unwrap();

    let snapshot = renderer.visual_snapshot().unwrap();

    assert_eq!(snapshot.nodes.len(), 2);
    assert_eq!(snapshot.edges.len(), 1);
    assert_eq!(snapshot.nodes[0].node_id, node_id);
    assert!(snapshot.nodes[0].highlighted);
    assert_eq!(snapshot.nodes[0].fill_color, "#1f6f78");
    assert_eq!(snapshot.edges[0].edge_id, edge_id);
    assert!(snapshot.edges[0].highlighted);
    assert_eq!(snapshot.edges[0].stroke_color, "#f2c94c");
}

#[cfg(feature = "native_render")]
#[test]
fn native_render_plugin_records_borderless_window_intent() {
    use bevy::prelude::{Plugin, With};

    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.update();

    let scene = app
        .world()
        .resource::<BibleGraphNativeRendererWindowScene>();

    assert!(
        app.world()
            .resource::<BibleGraphNativeRenderConfig>()
            .borderless_window
    );
    assert_eq!(scene.background_color, "#11151d");
    assert_eq!(scene.grid_color, "#253041");
    assert_eq!(scene.accent_color, "#f2c94c");
    assert_eq!(
        app.world()
            .resource::<BibleGraphNativeRendererWindowStatus>()
            .camera_count,
        1
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeCamera>>()
            .iter(app.world())
            .count(),
        1
    );
}

#[cfg(feature = "native_render")]
#[test]
fn native_window_runner_config_records_minimal_smoke_window_intent() {
    use std::num::NonZeroU64;

    let config = BibleGraphNativeWindowRunnerConfig::minimal_smoke(true);

    assert_eq!(config.title, "Eidetic Bible Graph");
    assert_eq!(config.width_px, 1280);
    assert_eq!(config.height_px, 720);
    assert!(config.borderless_window);
    assert!(config.run_on_any_thread);
    assert_eq!(config.auto_close_after_ms, None);

    let auto_close_ms = NonZeroU64::new(250).unwrap();
    let config = config.with_auto_close_after_ms(auto_close_ms);

    assert_eq!(config.auto_close_after_ms, Some(auto_close_ms));
}

#[cfg(feature = "native_render")]
#[test]
fn native_window_control_handle_records_close_requests() {
    let control = BibleGraphNativeWindowControlHandle::new();

    assert!(!control.close_requested());
    assert!(!control.ready());
    assert!(!control.visible());

    control.request_close();
    control.mark_ready();
    control.mark_visible(true);

    assert!(control.close_requested());
    assert!(control.ready());
    assert!(control.visible());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_app_installs_close_control_resource() {
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    configure_controlled_minimal_bible_graph_native_window_app(
        &mut app,
        BibleGraphNativeWindowRunnerConfig::minimal_smoke(true),
        control.clone(),
    );

    assert!(
        app.world()
            .contains_resource::<BibleGraphNativeWindowControl>()
    );
    assert!(!control.close_requested());
    assert!(!control.ready());

    app.update();

    assert!(control.ready());
    assert!(control.visible());

    control.request_close();

    assert!(control.close_requested());
}

#[cfg(feature = "native_render")]
#[test]
fn renderer_app_can_start_as_renderer_window_consumer() {
    let renderer = BibleGraphRendererApp::new_renderer_window();

    assert!(renderer.renderer_window_ready());
}

#[cfg(feature = "native_render")]
#[test]
fn renderer_window_records_physical_bounds() {
    let mut renderer = BibleGraphRendererApp::new_renderer_window();

    renderer.set_renderer_window_bounds(1280, 720);

    let bounds = renderer.renderer_window_bounds();
    assert_eq!(bounds.width_px, 1280);
    assert_eq!(bounds.height_px, 720);
}

#[cfg(feature = "native_render")]
#[test]
fn renderer_window_rebuilds_projection_visual_entities() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId::new();
    let mut renderer = BibleGraphRendererApp::new_renderer_window();

    renderer
        .set_projection(projection_with_influence(node_id, edge_id, influence_id))
        .unwrap();

    assert_eq!(renderer.native_visual_counts(), (2, 1));
}

#[test]
fn renderer_app_rejects_unknown_influence_selection() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let unknown_influence_id = ContextInfluenceId::new();
    let mut renderer = BibleGraphRendererApp::new();
    renderer
        .set_projection(projection_with_node(node_id))
        .unwrap();

    assert_eq!(
        renderer.select_influence(unknown_influence_id),
        Err(BibleGraphRendererError::UnknownInfluence {
            influence_id: unknown_influence_id
        })
    );
}

fn projection_with_node(node_id: BibleGraphNodeId) -> BibleRenderGraphProjection {
    BibleRenderGraphProjection {
        focused_root_id: None,
        selected_node_id: None,
        selected_timeline_node_id: None,
        nodes: vec![BibleRenderGraphNode {
            node_id,
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            label: "Ada".to_string(),
            system_owned: false,
            sort_order: 0,
            depth: 0,
            position: BibleRenderGraphPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }],
        edges: Vec::new(),
        neighborhoods: Vec::new(),
        influences: Vec::new(),
    }
}

fn projection_with_node_count(node_count: usize) -> BibleRenderGraphProjection {
    BibleRenderGraphProjection {
        focused_root_id: None,
        selected_node_id: None,
        selected_timeline_node_id: None,
        nodes: (0..node_count)
            .map(|index| BibleRenderGraphNode {
                node_id: BibleGraphNodeId::new(format!("node.test.{index}")).unwrap(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                label: format!("Node {index}"),
                system_owned: false,
                sort_order: u32::try_from(index).unwrap_or(u32::MAX),
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: index as f32,
                    y: 0.0,
                    z: 0.0,
                },
            })
            .collect(),
        edges: Vec::new(),
        neighborhoods: Vec::new(),
        influences: Vec::new(),
    }
}

fn projection_with_edge(source_id: BibleGraphNodeId) -> BibleRenderGraphProjection {
    let target_id = BibleGraphNodeId::new("node.place.beach").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    BibleRenderGraphProjection {
        focused_root_id: None,
        selected_node_id: None,
        selected_timeline_node_id: None,
        nodes: vec![
            BibleRenderGraphNode {
                node_id: source_id.clone(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                label: "Ada".to_string(),
                system_owned: false,
                sort_order: 0,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            BibleRenderGraphNode {
                node_id: target_id.clone(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("place").unwrap(),
                label: "Beach".to_string(),
                system_owned: false,
                sort_order: 1,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 0.0,
                    y: 150.0,
                    z: 0.0,
                },
            },
        ],
        edges: vec![BibleRenderGraphEdge {
            edge_id: edge_id.clone(),
            from_node_id: source_id.clone(),
            to_node_id: target_id.clone(),
            edge_kind: BibleGraphEdgeKind::LocatedIn,
            label: "located in".to_string(),
            directed: true,
            sort_order: 0,
        }],
        neighborhoods: vec![
            BibleRenderGraphNeighborhood {
                node_id: source_id,
                connected_node_ids: vec![target_id.clone()],
                edge_ids: vec![edge_id.clone()],
            },
            BibleRenderGraphNeighborhood {
                node_id: target_id,
                connected_node_ids: Vec::new(),
                edge_ids: vec![edge_id],
            },
        ],
        influences: Vec::new(),
    }
}

fn projection_with_influence(
    source_id: BibleGraphNodeId,
    edge_id: BibleGraphEdgeId,
    influence_id: ContextInfluenceId,
) -> BibleRenderGraphProjection {
    let mut projection = projection_with_edge(source_id.clone());
    projection.edges[0].edge_id = edge_id.clone();
    projection.neighborhoods[0].edge_ids = vec![edge_id.clone()];
    projection.neighborhoods[1].edge_ids = vec![edge_id.clone()];
    projection.influences = vec![BibleRenderGraphInfluence {
        influence_id,
        timeline_node_id: NodeId::new(),
        source_layer: StoryLevel::Scene,
        influence_kind: ContextInfluenceKind::Direct,
        confidence: 0.9,
        reason: "Scene uses Ada at the beach.".to_string(),
        provenance: ContextInfluenceProvenance::AiSelected,
        bible_node_id: Some(source_id),
        bible_edge_id: Some(edge_id),
        sort_order: 1,
    }];
    projection
}
