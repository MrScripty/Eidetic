use super::*;
#[cfg(feature = "native_render")]
use crate::native_render::{
    BibleGraphNativeLabelOverlayCamera, BibleGraphNativeMaterial,
    BibleGraphNativeSelectionOutlineBillboard, BibleGraphNativeSelectionOutlineVisual,
};
use eidetic_core::contracts::{
    BibleGraphEdgeKind, BibleGraphNodeCategory, BibleGraphSchemaKey, BibleRenderGraphEdge,
    BibleRenderGraphInfluence, BibleRenderGraphNode, BibleRenderGraphPosition,
    ContextInfluenceKind, ContextInfluenceProvenance,
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
fn renderer_app_emits_validated_focus_and_navigation_commands() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let mut renderer = BibleGraphRendererApp::new();
    renderer
        .set_projection(projection_with_node(node_id.clone()))
        .unwrap();

    assert_eq!(renderer.focus_node(node_id.clone()), Ok(()));
    assert_eq!(renderer.navigate_to_node(node_id.clone()), Ok(()));
    assert_eq!(renderer.delete_node(node_id.clone()), Ok(()));
    assert_eq!(renderer.create_connected_node(node_id.clone()), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![
            BibleGraphRendererCommand::FocusNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::NavigateToNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::DeleteNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::CreateConnectedNode { parent_id: node_id }
        ]
    );
}

#[test]
fn renderer_app_emits_clear_selection_without_projection_ownership() {
    let mut renderer = BibleGraphRendererApp::new();

    assert_eq!(renderer.clear_selection(), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![BibleGraphRendererCommand::ClearSelection]
    );
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
            active_timeline_ms: None,
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
    assert_eq!(snapshot.nodes[0].fill_color, "#6495ed");
    assert_eq!(snapshot.edges[0].edge_id, edge_id);
    assert!(snapshot.edges[0].highlighted);
    assert_eq!(snapshot.edges[0].stroke_color, "#f2c94c");
}

#[test]
fn renderer_app_derives_3d_structural_edges_from_parent_nodes() {
    let root_id = BibleGraphNodeId::new("canonical.characters").unwrap();
    let child_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let projection = BibleRenderGraphProjection {
        focused_root_id: None,
        selected_node_id: None,
        selected_timeline_node_id: None,
        active_timeline_ms: None,
        nodes: vec![
            BibleRenderGraphNode {
                node_id: root_id.clone(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("canonical.root.characters").unwrap(),
                category: BibleGraphNodeCategory::Character,
                label: "Characters".to_string(),
                system_owned: true,
                sort_order: 0,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 0.0,
                    y: 0.0,
                    z: -80.0,
                },
            },
            BibleRenderGraphNode {
                node_id: child_id.clone(),
                parent_id: Some(root_id.clone()),
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                category: BibleGraphNodeCategory::Character,
                label: "Ada".to_string(),
                system_owned: false,
                sort_order: 1,
                depth: 1,
                position: BibleRenderGraphPosition {
                    x: 320.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        ],
        edges: Vec::new(),
        neighborhoods: Vec::new(),
        influences: Vec::new(),
    };

    let snapshot = build_bible_graph_visual_3d_snapshot(&projection);

    assert_eq!(snapshot.nodes.len(), 2);
    assert_eq!(snapshot.edges.len(), 1);
    assert_eq!(
        snapshot.edges[0].edge_class,
        BibleGraphVisual3dEdgeClass::Structural
    );
    assert_eq!(snapshot.edges[0].from_node_id, root_id);
    assert_eq!(snapshot.edges[0].to_node_id, child_id);
    assert!(snapshot.nodes[0].label_visible);
}

#[test]
fn renderer_app_3d_visual_snapshot_highlights_selected_neighborhood() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let mut projection = projection_with_edge(node_id.clone());
    projection.selected_node_id = Some(node_id.clone());

    let snapshot = build_bible_graph_visual_3d_snapshot(&projection);

    assert_eq!(snapshot.edges.len(), 1);
    assert_eq!(
        snapshot.edges[0].edge_class,
        BibleGraphVisual3dEdgeClass::Semantic
    );
    assert_eq!(snapshot.edges[0].edge_id, edge_id);
    assert!(snapshot.edges[0].selected);
    assert!(snapshot.edges[0].highlighted);
    assert!(snapshot.nodes.iter().any(|node| node.selected));
    assert!(snapshot.nodes.iter().all(|node| node.label_visible));
    let selected_node = snapshot
        .nodes
        .iter()
        .find(|node| node.selected)
        .expect("selected node should be projected");
    assert_eq!(selected_node.label_font_size, 16.0);
    assert_eq!(selected_node.label_color, "#f6d977");
    assert!(
        snapshot
            .nodes
            .iter()
            .filter(|node| !node.selected)
            .all(|node| node.highlighted && !node.dimmed)
    );
}

#[test]
fn renderer_app_visual_snapshot_preserves_category_colors_under_highlight() {
    let node_id = BibleGraphNodeId::new("node.prop.lantern").unwrap();
    let influence_id = ContextInfluenceId::new();
    let mut projection = projection_with_node(node_id.clone());
    projection.nodes[0].schema_key = BibleGraphSchemaKey::new("prop").unwrap();
    projection.nodes[0].category = BibleGraphNodeCategory::Prop;
    projection.influences.push(BibleRenderGraphInfluence {
        influence_id,
        timeline_node_id: NodeId::new(),
        source_layer: StoryLevel::Scene,
        influence_kind: ContextInfluenceKind::Direct,
        confidence: 0.9,
        reason: "Lantern influence".to_string(),
        provenance: ContextInfluenceProvenance::AiSelected,
        bible_node_id: Some(node_id),
        bible_edge_id: None,
        sort_order: 1,
    });

    let snapshot = build_bible_graph_visual_3d_snapshot(&projection);

    assert!(snapshot.nodes[0].highlighted);
    assert_eq!(snapshot.nodes[0].fill_color, "#f97316");
    assert!(snapshot.nodes.iter().all(|node| node.label_visible));
    assert_eq!(snapshot.nodes[0].label_font_size, 14.0);
    assert_eq!(snapshot.nodes[0].label_color, "#c9f3f5");
}

#[test]
fn renderer_visual_snapshots_share_category_fill_colors() {
    let mut projection =
        projection_with_node(BibleGraphNodeId::new("node.location.archive").unwrap());
    projection.nodes[0].schema_key = BibleGraphSchemaKey::new("location").unwrap();
    projection.nodes[0].category = BibleGraphNodeCategory::Location;

    let visual_snapshot = build_bible_graph_visual_snapshot(&projection);
    let visual_3d_snapshot = build_bible_graph_visual_3d_snapshot(&projection);

    assert_eq!(visual_snapshot.nodes[0].fill_color, "#22c55e");
    assert_eq!(
        visual_snapshot.nodes[0].fill_color,
        visual_3d_snapshot.nodes[0].fill_color
    );
}

#[test]
fn renderer_app_3d_visual_snapshot_keeps_node_labels_visible() {
    let mut projection = projection_with_node_count(16);
    projection.nodes[0].system_owned = true;
    projection.nodes[0].label = "Characters".to_string();

    let snapshot = build_bible_graph_visual_3d_snapshot(&projection);

    assert_eq!(snapshot.nodes.len(), 16);
    assert!(snapshot.nodes.iter().all(|node| node.label_visible));
}

#[cfg(feature = "native_render")]
#[test]
fn native_render_feature_enables_visible_text_pipeline() {
    let manifest = include_str!("../Cargo.toml");

    assert!(manifest.contains("\"bevy/bevy_ui\","));
    assert!(manifest.contains("\"bevy/bevy_ui_render\","));
    assert!(manifest.contains("\"bevy/default_font\","));
}

#[cfg(feature = "native_render")]
#[test]
fn native_render_plugin_records_3d_window_scene_intent() {
    use bevy::prelude::{Plugin, With};

    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.update();

    let scene = app
        .world()
        .resource::<BibleGraphNativeRendererWindowScene>();

    assert!(
        !app.world()
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
        2
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::prelude::Camera3d>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::prelude::Camera2d>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeLabelOverlayCamera>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeCamera>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::prelude::PointLight>>()
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
    assert!(!config.borderless_window);
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

    control.request_close();

    assert!(control.close_requested());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_os_close_requests_shutdown() {
    let control = BibleGraphNativeWindowControlHandle::new();
    let window_control = BibleGraphNativeWindowControl::from(&control);

    control.mark_visible(true);
    window_control.request_close_from_os_window();

    assert!(control.close_requested());
    assert!(!control.visible());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_app_rebuilds_projection_visuals_from_control() {
    use bevy::prelude::{Plugin, With};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId::new();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_influence(node_id, edge_id, influence_id));

    app.update();

    assert_eq!(
        control.native_visual_counts(),
        BibleGraphNativeVisualStatus {
            node_count: 2,
            edge_count: 1
        }
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::prelude::Sprite>>()
            .iter(app.world())
            .count(),
        0
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::prelude::Mesh3d>>()
            .iter(app.world())
            .count(),
        3
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::prelude::MeshMaterial3d<BibleGraphNativeMaterial>>>()
            .iter(app.world())
            .count(),
        3
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::ui::prelude::Text>>()
            .iter(app.world())
            .count(),
        2
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeInfluenceVisual>>()
            .iter(app.world())
            .count(),
        1
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_uses_backend_category_material_colors() {
    use bevy::prelude::{Assets, Color, MeshMaterial3d, Plugin};

    let source_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_edge(source_id));

    app.update();

    let material_colors = {
        let world = app.world_mut();
        let mut query = world.query::<(
            &BibleGraphNativeNodeVisual,
            &MeshMaterial3d<BibleGraphNativeMaterial>,
        )>();
        query
            .iter(world)
            .map(|(node, material_handle)| {
                let material = world
                    .resource::<Assets<BibleGraphNativeMaterial>>()
                    .get(&material_handle.0)
                    .expect("node material should exist");
                (node.fill_color, material.color)
            })
            .collect::<Vec<_>>()
    };

    let character_color = Color::srgb(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0);
    let location_color = Color::srgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0);
    assert!(material_colors.contains(&("#6495ed", character_color.to_linear())));
    assert!(material_colors.contains(&("#22c55e", location_color.to_linear())));
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_renders_projection_derived_structural_edges() {
    use bevy::prelude::{Plugin, With};

    let child_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_parent_node(child_id));

    app.update();

    assert_eq!(
        control.native_visual_counts(),
        BibleGraphNativeVisualStatus {
            node_count: 2,
            edge_count: 1
        }
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeEdgeVisual>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<bevy::prelude::Mesh3d>>()
            .iter(app.world())
            .count(),
        3
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_uses_3d_ray_node_hit_testing() {
    use crate::native_render::nearest_native_node_on_ray;
    use bevy::prelude::{Dir3, Plugin, Ray3d, Vec3};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let target_id = BibleGraphNodeId::new("node.place.beach").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_edge(node_id.clone()));
    app.update();

    let mut nodes = app.world_mut().query::<&BibleGraphNativeNodeVisual>();
    assert_eq!(
        nearest_native_node_on_ray(
            nodes.iter(app.world()),
            Ray3d::new(Vec3::new(0.0, 0.0, 900.0), Dir3::NEG_Z)
        ),
        Some(node_id)
    );
    assert_eq!(
        nearest_native_node_on_ray(
            nodes.iter(app.world()),
            Ray3d::new(Vec3::new(0.0, 150.0, 900.0), Dir3::NEG_Z)
        ),
        Some(target_id)
    );
    assert_eq!(
        nearest_native_node_on_ray(
            nodes.iter(app.world()),
            Ray3d::new(Vec3::new(900.0, 900.0, 900.0), Dir3::NEG_Z)
        ),
        None
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_maps_background_click_to_clear_selection() {
    use crate::native_render::bible_graph_native_click_command;
    use bevy::prelude::{Dir3, Plugin, Ray3d, Vec3};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_edge(node_id.clone()));
    app.update();

    let nodes = app
        .world_mut()
        .query::<&BibleGraphNativeNodeVisual>()
        .iter(app.world())
        .cloned()
        .collect::<Vec<_>>();
    let edges = app
        .world_mut()
        .query::<&BibleGraphNativeEdgeVisual>()
        .iter(app.world())
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(
        bible_graph_native_click_command(
            nodes.iter(),
            edges.iter(),
            Ray3d::new(Vec3::new(0.0, 0.0, 900.0), Dir3::NEG_Z)
        ),
        BibleGraphRendererCommand::SelectNode { node_id }
    );
    assert_eq!(
        bible_graph_native_click_command(
            nodes.iter(),
            edges.iter(),
            Ray3d::new(Vec3::new(0.0, 75.0, 900.0), Dir3::NEG_Z)
        ),
        BibleGraphRendererCommand::SelectEdge {
            edge_id: BibleGraphEdgeId::new("edge.ada.beach").unwrap()
        }
    );
    assert_eq!(
        bible_graph_native_click_command(
            nodes.iter(),
            edges.iter(),
            Ray3d::new(Vec3::new(900.0, 900.0, 900.0), Dir3::NEG_Z)
        ),
        BibleGraphRendererCommand::ClearSelection
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_uses_3d_ray_semantic_edge_hit_testing() {
    use crate::native_render::nearest_selectable_native_edge_on_ray;
    use bevy::prelude::{Dir3, Plugin, Ray3d, Vec3};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_edge(node_id));
    app.update();

    let mut edges = app.world_mut().query::<&BibleGraphNativeEdgeVisual>();
    assert_eq!(
        nearest_selectable_native_edge_on_ray(
            edges.iter(app.world()),
            Ray3d::new(Vec3::new(0.0, 75.0, 900.0), Dir3::NEG_Z)
        ),
        Some(edge_id)
    );
    assert_eq!(
        nearest_selectable_native_edge_on_ray(
            edges.iter(app.world()),
            Ray3d::new(Vec3::new(220.0, 75.0, 900.0), Dir3::NEG_Z)
        ),
        None
    );
}

#[cfg(feature = "native_render")]
#[test]
fn native_edge_segment_transform_uses_full_3d_segment() {
    use crate::native_render::native_edge_segment_transform;
    use bevy::prelude::Vec3;

    let from = Vec3::new(10.0, 20.0, 30.0);
    let to = Vec3::new(10.0, 120.0, 330.0);
    let expected_direction = (to - from).normalize();

    let (length, transform) = native_edge_segment_transform(from, to);
    let rotated_local_x = transform.rotation * Vec3::X;

    assert!((length - from.distance(to)).abs() < 0.001);
    assert_eq!(transform.translation, Vec3::new(10.0, 70.0, 180.0));
    assert!((rotated_local_x.x - expected_direction.x).abs() < 0.001);
    assert!((rotated_local_x.y - expected_direction.y).abs() < 0.001);
    assert!((rotated_local_x.z - expected_direction.z).abs() < 0.001);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_does_not_pick_structural_edges_as_relationships() {
    use crate::native_render::nearest_selectable_native_edge_on_ray;
    use bevy::prelude::{Dir3, Plugin, Ray3d, Vec3};

    let child_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_parent_node(child_id));
    app.update();

    let mut edges = app.world_mut().query::<&BibleGraphNativeEdgeVisual>();
    assert_eq!(
        nearest_selectable_native_edge_on_ray(
            edges.iter(app.world()),
            Ray3d::new(Vec3::new(160.0, 0.0, 900.0), Dir3::NEG_Z)
        ),
        None
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_retains_selection_state_and_label_visibility() {
    use bevy::prelude::{Plugin, Visibility, With};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let unrelated_id = BibleGraphNodeId::new("node.object.lantern").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();
    let mut projection = projection_with_edge(node_id.clone());
    projection.selected_node_id = Some(node_id.clone());
    projection.nodes.push(BibleRenderGraphNode {
        node_id: unrelated_id.clone(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("object").unwrap(),
        category: BibleGraphNodeCategory::Prop,
        label: "Lantern".to_string(),
        system_owned: false,
        sort_order: 2,
        depth: 0,
        position: BibleRenderGraphPosition {
            x: 400.0,
            y: 400.0,
            z: 0.0,
        },
    });

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection);
    app.update();

    let node_states = app
        .world_mut()
        .query::<&BibleGraphNativeNodeVisual>()
        .iter(app.world())
        .map(|node| {
            (
                node.node_id.clone(),
                node.selected,
                node.highlighted,
                node.dimmed,
                node.label_visible,
            )
        })
        .collect::<Vec<_>>();

    assert!(
        node_states
            .iter()
            .any(|(id, selected, highlighted, dimmed, label_visible)| {
                id == &node_id && *selected && *highlighted && !*dimmed && *label_visible
            })
    );
    assert!(
        node_states
            .iter()
            .any(|(id, selected, highlighted, dimmed, label_visible)| {
                id == &unrelated_id && !*selected && !*highlighted && *dimmed && *label_visible
            })
    );
    assert!(
        app.world_mut()
            .query::<(&BibleGraphNativeNodeLabelVisual, &Visibility)>()
            .iter(app.world())
            .any(|(label, visibility)| {
                label.node_id == unrelated_id && visibility == &Visibility::Visible
            })
    );
    assert!(
        app.world_mut()
            .query::<&BibleGraphNativeEdgeVisual>()
            .iter(app.world())
            .any(|edge| edge.selected && edge.highlighted && !edge.dimmed)
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeSelectionOutlineVisual>>()
            .iter(app.world())
            .count(),
        1
    );
    assert!(
        app.world_mut()
            .query::<&BibleGraphNativeSelectionOutlineVisual>()
            .iter(app.world())
            .any(|outline| {
                outline.node_id == node_id
                    && outline.radius > 0.0
                    && outline.outline_color == "#f2c94c"
            })
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeSelectionOutlineBillboard>>()
            .iter(app.world())
            .count(),
        1
    );

    control.set_projection(projection_with_edge(node_id));
    app.update();

    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<BibleGraphNativeSelectionOutlineVisual>>()
            .iter(app.world())
            .count(),
        0
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_renders_node_labels_on_overlay_layer() {
    use bevy::prelude::{Plugin, Vec2, With};
    use bevy::ui::prelude::{IsDefaultUiCamera, Node, PositionType, Text, UiTargetCamera};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_node(node_id));
    app.update();

    let label_entities = {
        let mut label_entities = app
            .world_mut()
            .query_filtered::<(&Node, &Text, &UiTargetCamera), With<BibleGraphNativeLabelBillboard>>();
        label_entities.iter(app.world()).collect::<Vec<_>>()
    };
    let overlay_position = crate::native_render::native_node_label_overlay_position(
        Vec2::new(640.0, 360.0),
        Vec2::new(1280.0, 720.0),
        34.0,
    );

    assert_eq!(label_entities.len(), 1);
    assert_eq!(label_entities[0].0.position_type, PositionType::Absolute);
    assert_eq!(label_entities[0].1.0, "Ada");
    assert!(
        app.world()
            .get::<IsDefaultUiCamera>(label_entities[0].2.entity())
            .is_some()
    );
    assert_eq!(overlay_position.x, 640.0);
    assert!(overlay_position.y > 0.0);
    assert_ne!(overlay_position, Vec2::ZERO);
}

#[cfg(feature = "native_render")]
#[test]
fn native_camera_navigation_supports_pan_and_zoom_intents() {
    use crate::native_render::native_camera_navigation_delta;
    use bevy::prelude::Vec3;

    assert_eq!(
        native_camera_navigation_delta(false, false, false, false, false, false, 1.0),
        Vec3::ZERO
    );
    assert_eq!(
        native_camera_navigation_delta(true, false, false, false, false, false, 1.0),
        Vec3::new(-420.0, 0.0, 0.0)
    );
    assert_eq!(
        native_camera_navigation_delta(false, false, false, false, false, true, 1.0),
        Vec3::new(0.0, 0.0, -650.0)
    );
    assert_eq!(
        native_camera_navigation_delta(false, false, false, false, true, false, 0.5),
        Vec3::new(0.0, 0.0, 325.0)
    );
}

#[cfg(feature = "native_render")]
#[test]
fn native_camera_edge_pan_uses_viewport_bump_direction() {
    use crate::native_render::{native_camera_edge_pan_delta, native_camera_edge_pan_direction};
    use bevy::prelude::{Transform, Vec2, Vec3};

    let viewport = Vec2::new(800.0, 600.0);
    assert_eq!(
        native_camera_edge_pan_direction(Vec2::new(400.0, 300.0), viewport, 40.0),
        Vec2::ZERO
    );
    assert_eq!(
        native_camera_edge_pan_direction(Vec2::new(0.0, 300.0), viewport, 40.0),
        Vec2::NEG_X
    );
    assert_eq!(
        native_camera_edge_pan_direction(Vec2::new(800.0, 600.0), viewport, 40.0),
        Vec2::new(1.0, -1.0).normalize()
    );
    assert_eq!(
        native_camera_edge_pan_direction(Vec2::new(400.0, 0.0), viewport, 40.0),
        Vec2::Y
    );
    assert_eq!(
        native_camera_edge_pan_direction(Vec2::new(400.0, 600.0), viewport, 40.0),
        Vec2::NEG_Y
    );

    let front_camera = Transform::from_xyz(0.0, 0.0, 900.0).looking_at(Vec3::ZERO, Vec3::Y);
    assert_eq!(
        native_camera_edge_pan_delta(Vec2::X, &front_camera, 1.0),
        Vec3::new(520.0, 0.0, 0.0)
    );
    assert_eq!(
        native_camera_edge_pan_delta(Vec2::Y, &front_camera, 0.5),
        Vec3::new(0.0, 260.0, 0.0)
    );
}

#[cfg(feature = "native_render")]
#[test]
fn native_camera_frame_selected_moves_camera_over_selected_node() {
    use crate::native_render::native_camera_frame_selected_translation;
    use bevy::prelude::Vec3;

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let selected_node = BibleGraphNativeNodeVisual {
        node_id,
        x: 240.0,
        y: -120.0,
        z: 60.0,
        radius: 18.0,
        fill_color: "#1f6f78",
        outline_color: "#f2c94c",
        selected: true,
        highlighted: true,
        dimmed: false,
        label_visible: true,
    };

    assert_eq!(
        native_camera_frame_selected_translation(Vec3::new(10.0, 20.0, 900.0), &selected_node),
        Vec3::new(240.0, -120.0, 900.0)
    );
    assert_eq!(
        native_camera_frame_selected_translation(Vec3::new(10.0, 20.0, 120.0), &selected_node),
        Vec3::new(240.0, -120.0, 280.0)
    );
}

#[cfg(feature = "native_render")]
#[test]
fn native_camera_mouse_drag_and_scroll_helpers_match_requested_controls() {
    use crate::native_render::{
        native_camera_drag_orbit_transform, native_camera_drag_pan_delta,
        native_camera_scroll_zoom_delta, native_camera_view_orbit_target,
    };
    use bevy::prelude::{Transform, Vec2, Vec3};

    let front_camera = Transform::from_xyz(0.0, 0.0, 900.0).looking_at(Vec3::ZERO, Vec3::Y);
    assert_eq!(
        native_camera_drag_pan_delta(Vec2::new(20.0, -10.0), &front_camera),
        Vec3::new(-20.0, -10.0, 0.0)
    );
    assert_eq!(native_camera_scroll_zoom_delta(2.0), -160.0);

    let orbit_target = Vec3::ZERO;
    let mut transform = Transform::from_xyz(0.0, 0.0, 900.0).looking_at(orbit_target, Vec3::Y);
    native_camera_drag_orbit_transform(&mut transform, orbit_target, Vec2::new(20.0, 4.0));

    assert!(transform.translation.x < 0.0);
    assert!(transform.translation.y.abs() > 0.0);
    assert!(transform.translation.z > 0.0);

    let panned_camera = Transform::from_xyz(240.0, -120.0, 900.0)
        .looking_at(Vec3::new(240.0, -120.0, 0.0), Vec3::Y);
    assert_eq!(
        native_camera_view_orbit_target(&panned_camera),
        Some(Vec3::new(240.0, -120.0, 0.0))
    );

    let tilted_camera = Transform::from_xyz(0.0, -450.0, 900.0).looking_at(Vec3::ZERO, Vec3::Y);
    let vertical_pan = native_camera_drag_pan_delta(Vec2::new(0.0, 20.0), &tilted_camera);
    assert!(vertical_pan.y > 0.0);
    assert!(vertical_pan.z > 0.0);
}

#[cfg(feature = "native_render")]
#[test]
fn native_camera_recovery_maps_keyboard_intents_to_camera_commands() {
    use crate::native_render::native_camera_recovery_command;

    assert_eq!(native_camera_recovery_command(false, false), None);
    assert_eq!(
        native_camera_recovery_command(true, false),
        Some(BibleGraphCameraCommand::ResetCamera)
    );
    assert_eq!(
        native_camera_recovery_command(false, true),
        Some(BibleGraphCameraCommand::FitGraph)
    );
    assert_eq!(
        native_camera_recovery_command(true, true),
        Some(BibleGraphCameraCommand::ResetCamera)
    );
}

#[cfg(feature = "native_render")]
#[test]
fn renderer_app_applies_validated_native_camera_commands() {
    use bevy::prelude::{Transform, Vec3, With};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId::new();
    let mut renderer = BibleGraphRendererApp::new_renderer_window();
    renderer
        .set_projection(projection_with_influence(
            node_id.clone(),
            edge_id.clone(),
            influence_id,
        ))
        .unwrap();

    renderer
        .apply_camera_command(BibleGraphCameraCommand::FrameNode {
            node_id: node_id.clone(),
        })
        .unwrap();
    assert_eq!(
        native_camera_translation(&mut renderer),
        Vec3::new(0.0, 0.0, 220.0)
    );

    renderer
        .apply_camera_command(BibleGraphCameraCommand::FrameEdge { edge_id })
        .unwrap();
    assert_eq!(
        native_camera_translation(&mut renderer),
        Vec3::new(0.0, 75.0, 308.0)
    );

    renderer
        .apply_camera_command(BibleGraphCameraCommand::FrameInfluence { influence_id })
        .unwrap();
    assert_eq!(
        native_camera_translation(&mut renderer),
        Vec3::new(0.0, 0.0, 220.0)
    );

    let missing_node_id = BibleGraphNodeId::new("node.character.missing").unwrap();
    assert_eq!(
        renderer.apply_camera_command(BibleGraphCameraCommand::FrameNode {
            node_id: missing_node_id.clone()
        }),
        Err(BibleGraphRendererError::UnknownNode {
            node_id: missing_node_id
        })
    );

    fn native_camera_translation(renderer: &mut BibleGraphRendererApp) -> Vec3 {
        let mut cameras = renderer
            .app
            .world_mut()
            .query_filtered::<&Transform, With<BibleGraphNativeCamera>>();
        cameras.single(renderer.app.world()).unwrap().translation
    }
}

#[cfg(feature = "native_render")]
#[test]
fn native_camera_orbit_rotates_camera_around_target() {
    use crate::native_render::native_camera_orbit_translation;
    use bevy::prelude::Vec3;
    use std::f32::consts::FRAC_PI_2;

    let target = Vec3::new(100.0, 20.0, 40.0);
    let next_translation =
        native_camera_orbit_translation(Vec3::new(100.0, -60.0, 340.0), target, FRAC_PI_2);

    assert!((next_translation.x - 400.0).abs() < 0.001);
    assert_eq!(next_translation.y, -60.0);
    assert!((next_translation.z - 40.0).abs() < 0.001);
}

#[cfg(feature = "native_render")]
#[test]
fn native_visual_state_color_brightens_selection_and_dims_unrelated_nodes() {
    use crate::native_render::native_visual_state_color;
    use bevy::prelude::Color;

    assert_eq!(
        native_visual_state_color("#1f6f78", true, false, false),
        Color::srgb(
            (31.0_f32 / 255.0) + 0.22,
            (111.0_f32 / 255.0) + 0.22,
            (120.0_f32 / 255.0) + 0.22
        )
    );
    assert_eq!(
        native_visual_state_color("#1f6f78", false, true, false),
        Color::srgb(
            (31.0_f32 / 255.0) + 0.22,
            (111.0_f32 / 255.0) + 0.22,
            (120.0_f32 / 255.0) + 0.22
        )
    );
    assert_eq!(
        native_visual_state_color("#1f6f78", false, false, true),
        Color::srgb(
            (31.0_f32 / 255.0) * 0.32,
            (111.0_f32 / 255.0) * 0.32,
            (120.0_f32 / 255.0) * 0.32
        )
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_node_selection_commands() {
    use bevy::prelude::Plugin;

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let missing_node_id = BibleGraphNodeId::new("node.character.missing").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_node(node_id.clone()));
    app.update();

    assert_eq!(
        emit_bible_graph_native_node_selection(app.world_mut(), node_id.clone()),
        Ok(())
    );
    assert_eq!(
        control.drain_commands(),
        vec![BibleGraphRendererCommand::SelectNode { node_id }]
    );
    assert_eq!(
        emit_bible_graph_native_node_selection(app.world_mut(), missing_node_id.clone()),
        Err(BibleGraphRendererError::UnknownNode {
            node_id: missing_node_id
        })
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_inspection_and_edge_commands() {
    use bevy::prelude::Plugin;

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let missing_edge_id = BibleGraphEdgeId::new("edge.missing").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_edge(node_id.clone()));
    app.update();

    assert_eq!(
        emit_bible_graph_native_node_inspection(app.world_mut(), node_id.clone()),
        Ok(())
    );
    assert_eq!(
        emit_bible_graph_native_edge_selection(app.world_mut(), edge_id.clone()),
        Ok(())
    );
    assert_eq!(
        control.drain_commands(),
        vec![
            BibleGraphRendererCommand::InspectNode { node_id },
            BibleGraphRendererCommand::SelectEdge { edge_id }
        ]
    );
    assert_eq!(
        emit_bible_graph_native_edge_selection(app.world_mut(), missing_edge_id.clone()),
        Err(BibleGraphRendererError::UnknownEdge {
            edge_id: missing_edge_id
        })
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_focus_and_navigation_commands() {
    use bevy::prelude::Plugin;

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_node(node_id.clone()));
    app.update();

    assert_eq!(
        emit_bible_graph_native_node_focus(app.world_mut(), node_id.clone()),
        Ok(())
    );
    assert_eq!(
        emit_bible_graph_native_node_navigation(app.world_mut(), node_id.clone()),
        Ok(())
    );
    assert_eq!(
        emit_bible_graph_native_node_delete(app.world_mut(), node_id.clone()),
        Ok(())
    );
    assert_eq!(
        emit_bible_graph_native_connected_node_create(app.world_mut(), node_id.clone()),
        Ok(())
    );
    assert_eq!(
        control.drain_commands(),
        vec![
            BibleGraphRendererCommand::FocusNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::NavigateToNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::DeleteNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::CreateConnectedNode { parent_id: node_id }
        ]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_influence_selection_commands() {
    use bevy::prelude::Plugin;

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId::new();
    let missing_influence_id = ContextInfluenceId::new();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_influence(node_id, edge_id, influence_id));
    app.update();

    assert_eq!(
        emit_bible_graph_native_influence_selection(app.world_mut(), influence_id),
        Ok(())
    );
    assert_eq!(
        control.drain_commands(),
        vec![BibleGraphRendererCommand::SelectInfluence { influence_id }]
    );
    assert_eq!(
        emit_bible_graph_native_influence_selection(app.world_mut(), missing_influence_id),
        Err(BibleGraphRendererError::UnknownInfluence {
            influence_id: missing_influence_id
        })
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_clear_selection_command() {
    use bevy::prelude::Plugin;

    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));

    assert_eq!(
        emit_bible_graph_native_clear_selection(app.world_mut()),
        Ok(())
    );
    assert_eq!(
        control.drain_commands(),
        vec![BibleGraphRendererCommand::ClearSelection]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_camera_commands_are_bounded_and_drained() {
    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let control = BibleGraphNativeWindowControlHandle::new();

    for _ in 0..BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY {
        assert_eq!(
            control.push_camera_command(BibleGraphCameraCommand::FrameNode {
                node_id: node_id.clone()
            }),
            Ok(())
        );
    }

    assert_eq!(
        control.push_camera_command(BibleGraphCameraCommand::FrameNode {
            node_id: node_id.clone()
        }),
        Err(BibleGraphRendererError::CommandQueueFull)
    );
    assert_eq!(
        control.drain_camera_commands(),
        vec![
            BibleGraphCameraCommand::FrameNode {
                node_id: node_id.clone()
            };
            BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY
        ]
    );
    assert!(control.drain_camera_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn native_visual_rebuild_reuses_keyed_entities_and_removes_stale_entities() {
    use bevy::prelude::{Assets, Entity, Mesh, Plugin};

    let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId::new();
    let control = BibleGraphNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    BibleGraphNativeRenderPlugin.build(&mut app);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control));
    control.set_projection(projection_with_influence(
        node_id.clone(),
        edge_id.clone(),
        influence_id,
    ));
    app.update();

    let node_entity = native_node_entity(app.world_mut(), &node_id).unwrap();
    let node_label_entity = native_node_label_entity(app.world_mut(), &node_id).unwrap();
    let edge_entity = native_edge_entity(app.world_mut(), &edge_id).unwrap();
    let influence_entity = native_influence_entity(app.world_mut(), influence_id).unwrap();
    let first_asset_counts = native_asset_counts(app.world());

    control.set_projection(projection_with_influence(
        node_id.clone(),
        edge_id.clone(),
        influence_id,
    ));
    app.update();

    assert_eq!(
        native_node_entity(app.world_mut(), &node_id),
        Some(node_entity)
    );
    assert_eq!(
        native_node_label_entity(app.world_mut(), &node_id),
        Some(node_label_entity)
    );
    assert_eq!(
        native_edge_entity(app.world_mut(), &edge_id),
        Some(edge_entity)
    );
    assert_eq!(
        native_influence_entity(app.world_mut(), influence_id),
        Some(influence_entity)
    );
    assert_eq!(native_asset_counts(app.world()), first_asset_counts);

    control.set_projection(projection_with_node(node_id.clone()));
    app.update();

    assert_eq!(
        native_node_entity(app.world_mut(), &node_id),
        Some(node_entity)
    );
    assert_eq!(
        native_node_label_entity(app.world_mut(), &node_id),
        Some(node_label_entity)
    );
    assert_eq!(native_edge_entity(app.world_mut(), &edge_id), None);
    assert_eq!(native_influence_entity(app.world_mut(), influence_id), None);

    fn native_node_entity(
        world: &mut bevy::prelude::World,
        node_id: &BibleGraphNodeId,
    ) -> Option<Entity> {
        world
            .query::<(Entity, &BibleGraphNativeNodeVisual)>()
            .iter(world)
            .find_map(|(entity, node)| (&node.node_id == node_id).then_some(entity))
    }

    fn native_node_label_entity(
        world: &mut bevy::prelude::World,
        node_id: &BibleGraphNodeId,
    ) -> Option<Entity> {
        world
            .query::<(Entity, &BibleGraphNativeNodeLabelVisual)>()
            .iter(world)
            .find_map(|(entity, label)| (&label.node_id == node_id).then_some(entity))
    }

    fn native_edge_entity(
        world: &mut bevy::prelude::World,
        edge_id: &BibleGraphEdgeId,
    ) -> Option<Entity> {
        world
            .query::<(Entity, &BibleGraphNativeEdgeVisual)>()
            .iter(world)
            .find_map(|(entity, edge)| (&edge.edge_id == edge_id).then_some(entity))
    }

    fn native_influence_entity(
        world: &mut bevy::prelude::World,
        influence_id: ContextInfluenceId,
    ) -> Option<Entity> {
        world
            .query::<(Entity, &BibleGraphNativeInfluenceVisual)>()
            .iter(world)
            .find_map(|(entity, influence)| {
                (influence.influence_id == influence_id).then_some(entity)
            })
    }

    fn native_asset_counts(world: &bevy::prelude::World) -> (usize, usize) {
        (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<BibleGraphNativeMaterial>>().len(),
        )
    }
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
        active_timeline_ms: None,
        nodes: vec![BibleRenderGraphNode {
            node_id,
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            category: BibleGraphNodeCategory::Character,
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

#[cfg(feature = "native_render")]
fn projection_with_parent_node(child_id: BibleGraphNodeId) -> BibleRenderGraphProjection {
    let root_id = BibleGraphNodeId::new("canonical.characters").unwrap();
    BibleRenderGraphProjection {
        focused_root_id: None,
        selected_node_id: None,
        selected_timeline_node_id: None,
        active_timeline_ms: None,
        nodes: vec![
            BibleRenderGraphNode {
                node_id: root_id.clone(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("canonical.root.characters").unwrap(),
                category: BibleGraphNodeCategory::Character,
                label: "Characters".to_string(),
                system_owned: true,
                sort_order: 0,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 0.0,
                    y: 0.0,
                    z: -80.0,
                },
            },
            BibleRenderGraphNode {
                node_id: child_id,
                parent_id: Some(root_id),
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                category: BibleGraphNodeCategory::Character,
                label: "Ada".to_string(),
                system_owned: false,
                sort_order: 1,
                depth: 1,
                position: BibleRenderGraphPosition {
                    x: 320.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        ],
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
        active_timeline_ms: None,
        nodes: (0..node_count)
            .map(|index| BibleRenderGraphNode {
                node_id: BibleGraphNodeId::new(format!("node.test.{index}")).unwrap(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                category: BibleGraphNodeCategory::Character,
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
        active_timeline_ms: None,
        nodes: vec![
            BibleRenderGraphNode {
                node_id: source_id.clone(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                category: BibleGraphNodeCategory::Character,
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
                category: BibleGraphNodeCategory::Location,
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
