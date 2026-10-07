use super::*;

#[test]
fn deletion_restore_deletion_aba_refuses_preview_with_the_same_missing_arc() {
    let (mut conn, arc, _, b, _) = setup();
    delete(&mut conn, arc);
    let command = preview(&mut conn, &b);
    story_arc_command::record_create_story_arc_history(
        &mut conn,
        &CommandEnvelope::new(CreateStoryArcCommand {
            arc_id: arc,
            parent_arc_id: None,
            name: "Witness".into(),
            description: OLD.into(),
            arc_type: ArcType::APlot,
            color: Color::new(1, 2, 3),
        }),
        6,
    )
    .unwrap();
    delete(&mut conn, arc);
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(text(&conn, &b), b.text);
}

#[test]
fn both_saved_span_and_content_regeneration_locks_refuse_arc_acceptance() {
    for span_lock in [true, false] {
        let (mut conn, arc, _, b, _) = setup();
        change(&mut conn, arc, NEW);
        let command = preview(&mut conn, &b);
        if span_lock {
            let document = crate::script_store::load_document_projection(&conn, &b.document_id)
                .unwrap()
                .unwrap();
            let block = document
                .segments
                .iter()
                .flat_map(|segment| &segment.blocks)
                .find(|block| block.block.id == b.block_id)
                .unwrap();
            script_document_command::apply_set_script_lock(
                &mut conn,
                &CommandEnvelope::new(SetScriptLockCommand {
                    lock_id: ScriptLockId::new("lock.arc").unwrap(),
                    span_id: block.spans[0].id.clone(),
                    reason: "Keep authored text".into(),
                }),
                6,
            )
            .unwrap();
        } else {
            conn.execute(
                "UPDATE nodes SET locked=1 WHERE id=?1",
                [b.source_node_id.as_ref().unwrap()],
            )
            .unwrap();
        }
        assert!(accept(&mut conn, &command).is_err());
        assert_eq!(text(&conn, &b), b.text);
    }
}

#[tokio::test]
async fn targeted_provider_receives_current_arc_values_and_partial_failure_never_stores_a_proposal()
{
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, NEW);
    let command = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    let output =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains(NEW));
            assert!(prompt.user.contains("CURRENT TAGGED STORY ARC FIELDS"));
            // The proven input cause discloses historical consumption separately.
            assert!(prompt.user.contains(OLD));
            Ok(Box::pin(futures::stream::iter(vec![Ok(PREVIEW.into())]))
                as eidetic_core::ai::backend::GenerateStream)
        })
        .await
        .unwrap();
    assert_eq!(output, PREVIEW);
    let error = crate::script_impact_prompt::preview_with_provider(&binding, |_| async {
        Ok(Box::pin(futures::stream::iter(vec![
            Ok("Partial synthetic text".into()),
            Err(eidetic_core::Error::AiBackend(
                "Synthetic stream failure".into(),
            )),
        ])) as eidetic_core::ai::backend::GenerateStream)
    })
    .await
    .unwrap_err();
    assert!(error.to_string().contains("Synthetic stream failure"));
    assert_eq!(text(&conn, &b), b.text);
    assert!(
        crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
            .unwrap()
            .payload
            .proposals
            .is_empty()
    );
}
