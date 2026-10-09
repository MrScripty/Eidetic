use eidetic_core::ai::backend::GenerateStream;
use eidetic_core::contracts::*;
use futures::stream;
use rusqlite::Connection;

use crate::bible_graph_command as bible;
use crate::script_impact_review::{
    self,
    tests::{accept, edit, fixture, request, text},
};

fn node(conn: &mut Connection, id: &str, order: u32) {
    bible::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new(id).unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: id.into(),
            sort_order: order,
        }),
        1,
    )
    .unwrap();
}

fn field(conn: &mut Connection, key: &str, value: Option<&str>) {
    bible::apply_set_bible_graph_field(
        conn,
        &CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new("BeachHouse").unwrap(),
            part_id: BibleGraphPartId::new("house.profile").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            part_sort_order: 0,
            field_id: BibleGraphFieldId::new(format!("house.{key}")).unwrap(),
            field_key: BibleGraphFieldKey::new(key).unwrap(),
            value: value.map(|value| FieldValue::Text(value.into())),
            field_sort_order: 0,
        }),
        2,
    )
    .unwrap();
}

fn path(conn: &mut Connection, label: &str) {
    bible::apply_set_bible_graph_edge(
        conn,
        &CommandEnvelope::new(SetBibleGraphEdgeCommand {
            edge_id: BibleGraphEdgeId::new("Mara.house").unwrap(),
            from_node_id: BibleGraphNodeId::new("Mara").unwrap(),
            to_node_id: BibleGraphNodeId::new("BeachHouse").unwrap(),
            edge_kind: BibleGraphEdgeKind::LocatedIn,
            label: label.into(),
            directed: true,
            sort_order: 0,
        }),
        3,
    )
    .unwrap();
}

fn selection(conn: &Connection) -> ScriptRecallSelection {
    let query = BibleRecallRequest {
        anchor_node_id: BibleGraphNodeId::new("Mara").unwrap(),
        story_time_ms: None,
        direction: BibleRecallDirection::Both,
        edge_kinds: vec![],
        neighbor_limit: 8,
    };
    let recall = crate::bible_recall_projection::load(conn, &query)
        .unwrap()
        .payload;
    let house = recall
        .nodes
        .iter()
        .find(|node| node.node_id.as_str() == "BeachHouse")
        .unwrap();
    let fact = house
        .fields
        .iter()
        .find(|field| field.field_key.as_str() == "tagline")
        .unwrap();
    let BibleRecallFieldSource::Baseline {
        field_id,
        revision_event_id,
    } = &fact.source
    else {
        panic!("baseline required");
    };
    ScriptRecallSelection {
        query,
        facts: vec![ScriptRecallFactSelection {
            node_id: house.node_id.clone(),
            part_key: fact.part_key.clone(),
            field_key: fact.field_key.clone(),
            field_id: field_id.clone(),
            revision_event_id: *revision_event_id,
        }],
        names: recall
            .nodes
            .iter()
            .map(|node| BibleNodeNameInput {
                node_id: node.node_id.clone(),
                name: node.name.clone(),
                revision_event_id: node.name_revision_event_id.unwrap(),
            })
            .collect(),
        paths: recall.paths,
    }
}

fn setup() -> (
    Connection,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
    CommandEnvelope<RequestScriptImpactProposalCommand>,
) {
    let (mut conn, _, a, b, c) = fixture();
    node(&mut conn, "Mara", 0);
    for index in 0..205 {
        node(&mut conn, &format!("Filler{index:03}"), index + 1);
    }
    node(&mut conn, "BeachHouse", 10000);
    field(&mut conn, "tagline", Some("SELECTED dry roof — 雨"));
    field(
        &mut conn,
        "motivation",
        Some("UNSELECTED brass key in cellar"),
    );
    path(&mut conn, "Mara's untimed home");
    edit(
        &mut conn,
        &b,
        "EXT. STATION - NIGHT\n\nExact manual B — preserve this.\n\n",
    );
    let mut command = request(&conn, &b);
    command.payload.recall_selection = Some(selection(&conn));
    (conn, a, b, c, command)
}

fn propose(conn: &mut Connection, command: &CommandEnvelope<RequestScriptImpactProposalCommand>) {
    let binding = script_impact_review::capture(conn, &command.payload).unwrap();
    script_impact_review::record_proposal(
        conn,
        command,
        binding,
        "Synthetic selected-fact preview\n\n".into(),
        30,
    )
    .unwrap();
}

fn mutate(conn: &mut Connection, kind: &str) {
    match kind {
        "field" => field(conn, "tagline", Some("Changed roof")),
        "field ABA" => {
            field(conn, "tagline", Some("Changed roof"));
            field(conn, "tagline", Some("SELECTED dry roof — 雨"));
        }
        "clear" => field(conn, "tagline", None),
        "name ABA" | "name" => {
            bible::apply_set_bible_graph_node_name(
                conn,
                &CommandEnvelope::new(SetBibleGraphNodeNameCommand {
                    node_id: BibleGraphNodeId::new("BeachHouse").unwrap(),
                    name: "Renamed house".into(),
                }),
                35,
            )
            .unwrap();
            if kind == "name ABA" {
                bible::apply_set_bible_graph_node_name(
                    conn,
                    &CommandEnvelope::new(SetBibleGraphNodeNameCommand {
                        node_id: BibleGraphNodeId::new("BeachHouse").unwrap(),
                        name: "BeachHouse".into(),
                    }),
                    36,
                )
                .unwrap();
            }
        }
        "path" => path(conn, "Different untimed home"),
        "path ABA" => {
            path(conn, "Different untimed home");
            path(conn, "Mara's untimed home");
        }
        "path delete" => {
            bible::apply_delete_bible_graph_edge(
                conn,
                &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
                    edge_id: BibleGraphEdgeId::new("Mara.house").unwrap(),
                }),
                35,
            )
            .unwrap();
        }
        "node delete" => {
            bible::apply_delete_bible_graph_edge(
                conn,
                &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
                    edge_id: BibleGraphEdgeId::new("Mara.house").unwrap(),
                }),
                34,
            )
            .unwrap();
            bible::apply_delete_bible_graph_node(
                conn,
                &CommandEnvelope::new(DeleteBibleGraphNodeCommand {
                    node_id: BibleGraphNodeId::new("BeachHouse").unwrap(),
                }),
                35,
            )
            .unwrap();
        }
        "omitted" => field(
            conn,
            "tagline",
            Some(&"Oversized selected field ".repeat(3000)),
        ),
        "snapshot" => {
            bible::apply_set_bible_graph_snapshot_field(
                conn,
                &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
                    node_id: BibleGraphNodeId::new("BeachHouse").unwrap(),
                    snapshot_id: BibleGraphSnapshotId::new("house.future").unwrap(),
                    at_ms: 1000,
                    label: "Future".into(),
                    snapshot_sort_order: 0,
                    field_id: BibleGraphSnapshotFieldId::new("house.future.tagline").unwrap(),
                    part_key: BibleGraphPartKey::new("profile").unwrap(),
                    part_name: "Profile".into(),
                    field_key: BibleGraphFieldKey::new("tagline").unwrap(),
                    value: Some(FieldValue::Text("Timed roof".into())),
                    field_sort_order: 0,
                }),
                35,
            )
            .unwrap();
        }
        _ => panic!("unknown mutation"),
    }
}

#[tokio::test]
async fn selected_far_fact_and_only_its_supplemental_values_reach_the_actual_prompt() {
    let (conn, _, b, _, command) = setup();
    let mut ordinary = command.payload.clone();
    ordinary.recall_selection = None;
    let initial = script_impact_review::capture(&conn, &ordinary).unwrap();
    assert!(
        !initial
            .bible_context
            .payload
            .nodes
            .iter()
            .any(|node| node.node_id.as_str() == "BeachHouse")
    );
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    let chosen = binding
        .bible_inputs
        .iter()
        .find(|input| input.field_id.as_str() == "house.tagline")
        .unwrap();
    assert_eq!(
        chosen.revision_event_id,
        command.payload.recall_selection.as_ref().unwrap().facts[0].revision_event_id
    );
    assert!(
        !binding
            .bible_inputs
            .iter()
            .any(|input| input.field_id.as_str() == "house.motivation")
    );
    let output =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains("SELECTED dry roof — 雨"));
            assert!(!prompt.user.contains("UNSELECTED brass key in cellar"));
            assert!(prompt.user.contains("BeachHouse"));
            assert!(prompt.user.contains("Mara's untimed home"));
            assert!(prompt.user.contains("Exact manual B — preserve this."));
            assert!(
                prompt
                    .system
                    .contains("presentation placement is not fictional time")
            );
            let tokens: GenerateStream =
                Box::pin(stream::iter(vec![Ok("Synthetic preview".into())]));
            Ok(tokens)
        })
        .await
        .unwrap();
    assert_eq!(output, "Synthetic preview");
    assert_eq!(
        text(&conn, &b),
        "EXT. STATION - NIGHT\n\nExact manual B — preserve this.\n\n"
    );
}

#[test]
fn empty_selection_is_exact_existing_capture_behavior() {
    let (conn, _, _, _, mut command) = setup();
    command.payload.recall_selection = None;
    let old = script_impact_review::capture(&conn, &command.payload).unwrap();
    let mut empty = selection(&conn);
    empty.facts.clear();
    command.payload.recall_selection = Some(empty);
    let mut current = script_impact_review::capture(&conn, &command.payload).unwrap();
    current.request.recall_selection = None;
    assert_eq!(current, old);
}

#[test]
fn selection_refuses_over_limit_duplicates_unknown_omitted_temporal_and_forged_receipts() {
    let (conn, _, _, _, command) = setup();
    for invalid in [
        "over limit",
        "duplicate",
        "missing field",
        "timed query",
        "timed preview",
        "missing name",
        "missing path",
        "forged revision",
        "forged name",
        "forged path",
    ] {
        let mut request = command.payload.clone();
        let selection = request.recall_selection.as_mut().unwrap();
        match invalid {
            "over limit" => selection.facts = vec![selection.facts[0].clone(); 9],
            "duplicate" => selection.facts.push(selection.facts[0].clone()),
            "missing field" => {
                selection.facts[0].field_key = BibleGraphFieldKey::new("omitted").unwrap()
            }
            "timed query" => selection.query.story_time_ms = Some(1000),
            "timed preview" => request.story_time_ms = Some(1000),
            "missing name" => selection.names.clear(),
            "missing path" => selection.paths.clear(),
            "forged revision" => {
                selection.facts[0].revision_event_id = ChangeEventId(uuid::Uuid::new_v4())
            }
            "forged name" => selection.names[0].name = "Forged".into(),
            "forged path" => selection.paths[0].relationship.edge.label = "Forged".into(),
            _ => unreachable!(),
        }
        assert!(
            script_impact_review::capture(&conn, &request).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn source_edits_aba_deletion_and_temporal_reclassification_refuse_capture_and_late_provider_recording()
 {
    for kind in [
        "field",
        "field ABA",
        "clear",
        "name",
        "name ABA",
        "path",
        "path ABA",
        "path delete",
        "node delete",
        "snapshot",
        "omitted",
    ] {
        let (mut conn, _, b, _, command) = setup();
        let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
        let before_text = text(&conn, &b);
        mutate(&mut conn, kind);
        assert!(
            script_impact_review::capture(&conn, &command.payload).is_err(),
            "{kind}"
        );
        assert!(
            script_impact_review::record_proposal(
                &mut conn,
                &command,
                binding,
                "Synthetic delayed provider".into(),
                40
            )
            .is_err(),
            "{kind}"
        );
        assert!(
            crate::propagation_proposal_store::load_propagation_proposal(
                &conn,
                &command.payload.proposal_id
            )
            .unwrap()
            .is_none(),
            "{kind}"
        );
        assert_eq!(text(&conn, &b), before_text, "{kind}");
    }
}

#[test]
fn source_mutations_refuse_acceptance_and_preserve_exact_pending_proposal_and_saved_scene() {
    for kind in [
        "field ABA",
        "clear",
        "name ABA",
        "path ABA",
        "path delete",
        "node delete",
        "snapshot",
        "omitted",
    ] {
        let (mut conn, _, b, _, command) = setup();
        propose(&mut conn, &command);
        let pending = crate::propagation_proposal_store::load_propagation_proposal(
            &conn,
            &command.payload.proposal_id,
        )
        .unwrap();
        let before_text = text(&conn, &b);
        mutate(&mut conn, kind);
        assert!(accept(&mut conn, &command).is_err(), "{kind}");
        assert_eq!(
            crate::propagation_proposal_store::load_propagation_proposal(
                &conn,
                &command.payload.proposal_id
            )
            .unwrap(),
            pending,
            "{kind}"
        );
        assert_eq!(text(&conn, &b), before_text, "{kind}");
    }
}

#[test]
fn explicit_acceptance_updates_only_target_and_subsequent_selected_fact_name_and_path_edits_derive_impact()
 {
    for kind in ["field", "name", "path"] {
        let (mut conn, a, b, c, command) = setup();
        let before = crate::script_store::load_document_projection(&conn, &b.document_id)
            .unwrap()
            .unwrap();
        let target_before = before
            .segments
            .iter()
            .find(|segment| segment.segment.id == b.segment_id)
            .unwrap()
            .segment
            .clone();
        propose(&mut conn, &command);
        assert_eq!(
            crate::script_store::load_document_projection(&conn, &b.document_id)
                .unwrap()
                .unwrap(),
            before
        );
        accept(&mut conn, &command).unwrap();
        assert_eq!(text(&conn, &b), "Synthetic selected-fact preview\n\n");
        assert_eq!(text(&conn, &a), text_from(&before, &a));
        assert_eq!(text(&conn, &c), text_from(&before, &c));
        let current = crate::script_store::load_document_projection(&conn, &b.document_id)
            .unwrap()
            .unwrap();
        let target = current
            .segments
            .iter()
            .find(|segment| segment.segment.id == b.segment_id)
            .unwrap();
        assert_eq!(target.segment, target_before);
        assert!(!target.impact.as_ref().unwrap().needs_review);
        field(&mut conn, "motivation", Some("Unselected value changed"));
        assert!(
            !crate::script_impact_projection::load_impact(&conn, &b.segment_id)
                .unwrap()
                .unwrap()
                .needs_review,
            "unselected value must not become consumed"
        );
        mutate(&mut conn, kind);
        let impact = crate::script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap();
        let expected = match kind {
            "field" => "bible_field",
            "name" => "bible_node",
            _ => "bible_edge",
        };
        assert!(impact.needs_review, "{kind}");
        assert!(
            impact
                .causes
                .iter()
                .any(|cause| serde_json::to_value(&cause.input).unwrap()["kind"] == expected),
            "{kind}"
        );
        let mut fresh = request(&conn, &b);
        fresh.payload.recall_selection = Some(selection(&conn));
        // Explicitly selecting fresh evidence can restore the omitted consumed
        // entity; no durable context link or automatic renderer change is needed.
        let refreshed = script_impact_review::capture(&conn, &fresh.payload).unwrap();
        script_impact_review::record_proposal(
            &mut conn,
            &fresh,
            refreshed,
            "Synthetic fresh selected preview".into(),
            50,
        )
        .unwrap();
        accept(&mut conn, &fresh).unwrap();
        assert!(
            !crate::script_impact_projection::load_impact(&conn, &b.segment_id)
                .unwrap()
                .unwrap()
                .needs_review
        );
    }
}

fn text_from(document: &ScriptDocumentProjection, block: &SetScriptBlockCommand) -> String {
    document
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|item| item.block.id == block.block_id)
        .unwrap()
        .block
        .text
        .clone()
}

#[test]
fn rejection_and_locked_target_preserve_manual_text_placement_and_previous_proposals() {
    let (mut conn, _, b, _, command) = setup();
    propose(&mut conn, &command);
    let before = crate::script_store::load_document_projection(&conn, &b.document_id)
        .unwrap()
        .unwrap();
    crate::propagation_proposal_review::record_reject_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(RejectPropagationProposalCommand {
            proposal_id: command.payload.proposal_id.clone(),
            reason: None,
        }),
        40,
    )
    .unwrap();
    assert_eq!(
        crate::script_store::load_document_projection(&conn, &b.document_id)
            .unwrap()
            .unwrap(),
        before
    );
    let prior = crate::propagation_proposal_store::load_propagation_proposal(
        &conn,
        &command.payload.proposal_id,
    )
    .unwrap();
    let mut fresh = request(&conn, &b);
    fresh.payload.recall_selection = Some(selection(&conn));
    propose(&mut conn, &fresh);
    assert_eq!(
        crate::propagation_proposal_store::load_propagation_proposal(
            &conn,
            &command.payload.proposal_id
        )
        .unwrap(),
        prior
    );
    let block = before
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|item| item.block.id == b.block_id)
        .unwrap();
    crate::script_document_command::apply_set_script_lock(
        &mut conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("lock.selected").unwrap(),
            span_id: block.spans[0].id.clone(),
            reason: "Preserve manual wording".into(),
        }),
        45,
    )
    .unwrap();
    let locked = crate::script_store::load_document_projection(&conn, &b.document_id).unwrap();
    assert!(accept(&mut conn, &fresh).is_err());
    assert_eq!(
        crate::script_store::load_document_projection(&conn, &b.document_id).unwrap(),
        locked
    );
}

#[test]
fn selected_preview_cannot_overwrite_an_existing_proposal_or_accept_after_manual_target_aba() {
    let (mut conn, _, b, _, command) = setup();
    propose(&mut conn, &command);
    let prior = crate::propagation_proposal_store::load_propagation_proposal(
        &conn,
        &command.payload.proposal_id,
    )
    .unwrap();
    let duplicate = CommandEnvelope::new(command.payload.clone());
    let binding = script_impact_review::capture(&conn, &duplicate.payload).unwrap();
    assert!(
        script_impact_review::record_proposal(
            &mut conn,
            &duplicate,
            binding,
            "Different synthetic output".into(),
            40
        )
        .is_err()
    );
    assert_eq!(
        crate::propagation_proposal_store::load_propagation_proposal(
            &conn,
            &command.payload.proposal_id
        )
        .unwrap(),
        prior
    );
    let original = text(&conn, &b);
    edit(&mut conn, &b, "Intervening manual target");
    edit(&mut conn, &b, &original);
    let current = crate::script_store::load_document_projection(&conn, &b.document_id).unwrap();
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(
        crate::script_store::load_document_projection(&conn, &b.document_id).unwrap(),
        current
    );
    assert_eq!(
        crate::propagation_proposal_store::load_propagation_proposal(
            &conn,
            &command.payload.proposal_id
        )
        .unwrap(),
        prior
    );
}

#[test]
fn selected_pending_preview_refuses_canonical_placement_aba_without_reverting_manual_text_or_ranges()
 {
    let (mut conn, mut project, a, b) = crate::timeline_script_placement::tests::fixture();
    edit(&mut conn, &a, "Changed placement fixture source");
    edit(&mut conn, &b, "Exact manual placement target — 雨\n\n");
    node(&mut conn, "Mara", 0);
    node(&mut conn, "BeachHouse", 1);
    field(&mut conn, "tagline", Some("SELECTED dry roof — 雨"));
    path(&mut conn, "Mara's untimed home");
    let mut command = request(&conn, &b);
    command.payload.recall_selection = Some(selection(&conn));
    propose(&mut conn, &command);
    let id = eidetic_core::timeline::node::NodeId(
        uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap(),
    );
    let range = project.timeline.node(id).unwrap().time_range;
    let manual = text(&conn, &b);
    for start in [range.start_ms + 100, range.start_ms] {
        crate::timeline_command_history::record_set_timeline_node_range_history(
            &mut conn,
            &project,
            &CommandEnvelope::new(SetTimelineNodeRangeCommand {
                node_id: id,
                start_ms: start,
                end_ms: range.end_ms,
                expected: None,
            }),
            35,
        )
        .unwrap();
        project.timeline.nodes = crate::timeline_node_store::load_nodes(&conn).unwrap();
    }
    let current = crate::script_store::load_document_projection(&conn, &b.document_id).unwrap();
    let pending = crate::propagation_proposal_store::load_propagation_proposal(
        &conn,
        &command.payload.proposal_id,
    )
    .unwrap();
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(
        crate::script_store::load_document_projection(&conn, &b.document_id).unwrap(),
        current
    );
    assert_eq!(text(&conn, &b), manual);
    assert_eq!(
        crate::propagation_proposal_store::load_propagation_proposal(
            &conn,
            &command.payload.proposal_id
        )
        .unwrap(),
        pending
    );
}
