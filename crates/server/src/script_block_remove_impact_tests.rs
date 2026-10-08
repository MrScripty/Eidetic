use super::*;
use crate::{ai_script_context, script_impact_projection, script_impact_review};
use eidetic_core::contracts::*;

#[test]
fn removal_invalidates_old_review_and_fresh_targeted_preview_requires_explicit_acceptance() {
    let (mut conn, _, a, b, c) = script_impact_review::tests::fixture();
    let old_request = script_impact_review::tests::request(&conn, &b);
    let old_binding = script_impact_review::capture(&conn, &old_request.payload).unwrap();
    script_impact_review::record_proposal(
        &mut conn,
        &old_request,
        old_binding,
        "Obsolete preview".into(),
        25,
    )
    .unwrap();
    let command = super::tests::removal(&conn, &a);
    apply_remove_script_block(&mut conn, &command, 30).unwrap();
    let after_remove = script_store::load_document_projection(&conn, &a.document_id).unwrap();
    assert!(script_impact_review::tests::accept(&mut conn, &old_request).is_err());
    assert_eq!(
        script_store::load_document_projection(&conn, &a.document_id).unwrap(),
        after_remove
    );
    let impact = script_impact_projection::load_impact(&conn, &b.segment_id)
        .unwrap()
        .unwrap();
    let cause = impact
        .causes
        .iter()
        .find(|cause| {
            cause.input
                == SemanticDependencyEndpoint::ScriptBlock {
                    block_id: a.block_id.clone(),
                }
        })
        .unwrap();
    assert_eq!(cause.reason, ScriptImpactReason::Deleted);
    assert!(cause.current_revision_event_id.is_none());
    assert!(
        cause
            .input_excerpt
            .as_deref()
            .unwrap()
            .contains("Original A")
    );
    assert!(
        !script_impact_projection::load_impact(&conn, &c.segment_id)
            .unwrap()
            .is_some_and(|impact| impact.needs_review)
    );
    let memory = ai_script_context::load_script_context(
        &conn,
        eidetic_core::timeline::node::NodeId(
            uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap(),
        ),
        b.segment_start_ms,
        b.segment_end_ms,
    )
    .unwrap();
    assert!(!memory.iter().any(|input| input.block_id == a.block_id));
    let fresh_request = script_impact_review::tests::request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &fresh_request.payload).unwrap();
    assert!(
        !binding
            .script_inputs
            .iter()
            .any(|input| input.block_id == a.block_id)
    );
    script_impact_review::record_proposal(
        &mut conn,
        &fresh_request,
        binding,
        "B no longer refers to the removed scene material.".into(),
        40,
    )
    .unwrap();
    assert_eq!(
        script_store::load_document_projection(&conn, &a.document_id).unwrap(),
        after_remove
    );
    assert_eq!(
        script_impact_review::tests::accept(&mut conn, &fresh_request).unwrap(),
        RecordChangeOutcome::Recorded
    );
    let after = script_store::load_document_projection(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    assert!(after.segments[0].blocks.is_empty());
    assert_eq!(
        after.segments[1].blocks[0].block.text,
        "B no longer refers to the removed scene material."
    );
    assert_eq!(after.segments[2].blocks[0].block.text, "Original C");
    assert!(
        !script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
}
