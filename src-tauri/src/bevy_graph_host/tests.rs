use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
use eidetic_core::contracts::{
    BibleGraphEdgeKind, BibleGraphNodeId, BibleRenderGraphEdge, BibleRenderGraphInfluence,
    BibleRenderGraphNeighborhood, BibleRenderGraphNode, BibleRenderGraphPosition,
    ContextInfluenceId, ContextInfluenceKind, ContextInfluenceProvenance,
};
use eidetic_core::timeline::node::StoryLevel;
use uuid::Uuid;

use super::{
    BibleGraphHostError, BibleGraphHostStatus, DesktopBibleGraphHost,
    DesktopBibleGraphRendererOwner,
};

#[test]
fn host_applies_projection_and_reports_scene_counts() {
    let mut host = DesktopBibleGraphHost::new();

    let status = host.set_projection(sample_projection()).unwrap();

    assert_eq!(
        status,
        BibleGraphHostStatus {
            running: true,
            node_count: 2,
            edge_count: 1,
            influence_count: 1,
            last_error: None,
        }
    );
}

#[test]
fn host_validates_renderer_commands_and_drains_them() {
    let mut host = DesktopBibleGraphHost::new();
    let projection = sample_projection();
    let node_id = projection.nodes[0].node_id.clone();
    let edge_id = projection.edges[0].edge_id.clone();
    let influence_id = projection.influences[0].influence_id;
    host.set_projection(projection).unwrap();

    host.select_node(node_id.clone()).unwrap();
    host.inspect_node(node_id.clone()).unwrap();
    host.select_edge(edge_id.clone()).unwrap();
    host.select_influence(influence_id).unwrap();

    assert_eq!(
        host.drain_commands(),
        vec![
            BibleGraphRendererCommand::SelectNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::InspectNode { node_id },
            BibleGraphRendererCommand::SelectEdge { edge_id },
            BibleGraphRendererCommand::SelectInfluence { influence_id },
        ]
    );
    assert!(host.drain_commands().is_empty());
}

#[test]
fn host_records_renderer_errors_without_panicking() {
    let mut host = DesktopBibleGraphHost::new();
    host.set_projection(sample_projection()).unwrap();
    let missing = BibleGraphNodeId::new("node.missing").unwrap();

    let error = host.select_node(missing).unwrap_err();

    assert!(matches!(error, BibleGraphHostError::Renderer(_)));
    assert!(host.status().last_error.is_some());
}

#[test]
fn host_stop_drops_renderer_state() {
    let mut host = DesktopBibleGraphHost::new();
    host.set_projection(sample_projection()).unwrap();

    let status = host.stop();

    assert_eq!(
        status,
        BibleGraphHostStatus {
            running: false,
            node_count: 0,
            edge_count: 0,
            influence_count: 0,
            last_error: None,
        }
    );
}

#[test]
fn owner_runs_renderer_on_dedicated_thread() {
    let owner = DesktopBibleGraphRendererOwner::start().unwrap();
    let projection = sample_projection();

    let status = owner.set_projection(projection).unwrap();

    assert_eq!(status.node_count, 2);
    assert_eq!(status.edge_count, 1);
    assert_eq!(status.influence_count, 1);
    assert!(status.running);
    owner.stop().unwrap();
}

#[test]
fn owner_drains_validated_renderer_commands() {
    let owner = DesktopBibleGraphRendererOwner::start().unwrap();
    let projection = sample_projection();
    let node_id = projection.nodes[0].node_id.clone();
    let edge_id = projection.edges[0].edge_id.clone();
    let influence_id = projection.influences[0].influence_id;
    owner.set_projection(projection).unwrap();

    owner.select_node(node_id.clone()).unwrap();
    owner.inspect_node(node_id.clone()).unwrap();
    owner.select_edge(edge_id.clone()).unwrap();
    owner.select_influence(influence_id).unwrap();

    assert_eq!(
        owner.drain_commands().unwrap(),
        vec![
            BibleGraphRendererCommand::SelectNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::InspectNode { node_id },
            BibleGraphRendererCommand::SelectEdge { edge_id },
            BibleGraphRendererCommand::SelectInfluence { influence_id },
        ]
    );
    assert!(owner.drain_commands().unwrap().is_empty());
    owner.stop().unwrap();
}

#[test]
fn owner_reports_stopped_after_shutdown() {
    let owner = DesktopBibleGraphRendererOwner::start().unwrap();
    owner.stop().unwrap();

    let error = owner.status().unwrap_err();

    assert_eq!(error, BibleGraphHostError::OwnerStopped);
}

fn sample_projection() -> eidetic_core::contracts::BibleRenderGraphProjection {
    let ada_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let beach_id = BibleGraphNodeId::new("node.location.beach").unwrap();
    let edge_id = eidetic_core::contracts::BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId(Uuid::from_u128(1));
    let timeline_node_id = eidetic_core::timeline::node::NodeId(Uuid::from_u128(2));

    eidetic_core::contracts::BibleRenderGraphProjection {
        nodes: vec![
            BibleRenderGraphNode {
                node_id: ada_id.clone(),
                parent_id: None,
                schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("character").unwrap(),
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
                node_id: beach_id.clone(),
                parent_id: None,
                schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("location").unwrap(),
                label: "Beach".to_string(),
                system_owned: false,
                sort_order: 1,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        ],
        edges: vec![BibleRenderGraphEdge {
            edge_id: edge_id.clone(),
            from_node_id: ada_id.clone(),
            to_node_id: beach_id.clone(),
            edge_kind: BibleGraphEdgeKind::LocatedIn,
            label: "located in".to_string(),
            directed: true,
            sort_order: 0,
        }],
        neighborhoods: vec![BibleRenderGraphNeighborhood {
            node_id: ada_id.clone(),
            connected_node_ids: vec![beach_id],
            edge_ids: vec![edge_id.clone()],
        }],
        influences: vec![BibleRenderGraphInfluence {
            influence_id,
            timeline_node_id,
            source_layer: StoryLevel::Scene,
            influence_kind: ContextInfluenceKind::Direct,
            confidence: 0.9,
            reason: "Scene uses Ada at the beach.".to_string(),
            provenance: ContextInfluenceProvenance::AiSelected,
            bible_node_id: Some(ada_id),
            bible_edge_id: Some(edge_id),
            sort_order: 0,
        }],
    }
}
