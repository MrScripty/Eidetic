//! Derive manual-edit eligibility from owned history and actual generation dependencies.
use crate::history_store::HistoryStoreError;
use eidetic_core::contracts::*;
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) fn dependencies(
    conn: &Connection,
    segment: &ScriptSegmentId,
    generation: ChangeEventId,
) -> Result<Vec<SemanticDependency>, HistoryStoreError> {
    use crate::semantic_dependency_store::{
        DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
    };
    Ok(
        crate::semantic_dependency_store::load_semantic_dependency_projection(
            conn,
            &SemanticDependencyFilter {
                endpoint: DependencyEndpointFilter {
                    kind: "script_segment".into(),
                    id: segment.as_str().into(),
                    part_key: None,
                    field_key: None,
                },
                direction: DependencyDirection::Source,
            },
        )
        .map_err(|e| HistoryStoreError::InvalidValue(e.to_string()))?
        .payload
        .dependencies
        .into_iter()
        .filter(|d| {
            d.revision_binding
                .as_ref()
                .is_some_and(|b| b.source_revision_event_id == generation)
        })
        .collect(),
    )
}

pub(crate) fn evidence(
    conn: &Connection,
    segment: &ScriptSegmentId,
    generation: ChangeEventId,
    block: &str,
    dependencies: &[SemanticDependency],
) -> Result<Option<ScriptFactEditEvidence>, HistoryStoreError> {
    // No script projection read here: this function is part of that projection owner.
    let row = conn.query_row("SELECT s.document_id, b.updated_event_id, s.updated_event_id, b.text, s.start_ms, s.end_ms, c.payload_json
        FROM script_blocks b JOIN script_segments s ON s.id=b.segment_id JOIN script_documents d ON d.id=s.document_id
        JOIN change_events e ON e.id=b.updated_event_id JOIN commands c ON c.id=e.command_id
        WHERE b.id=?1 AND s.id=?2 AND b.deleted_event_id IS NULL AND s.deleted_event_id IS NULL AND d.deleted_event_id IS NULL
        AND c.payload_type='script.edit_block'", params![block,segment.as_str()], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,String>(3)?, r.get::<_,u64>(4)?, r.get::<_,u64>(5)?, r.get::<_,String>(6)?))).optional()?;
    let Some((document, revision, segment_revision, text, start_ms, end_ms, command)) = row else {
        return Ok(None);
    };
    let Ok(edit) = serde_json::from_str::<EditScriptBlockCommand>(&command) else {
        return Ok(None);
    };
    if edit.block_id.as_str() != block || edit.document_id.as_str() != document || edit.text != text
    {
        return Ok(None);
    }
    let before = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::ScriptBlock,
        block,
        edit.expected_revision_event_id,
    )?;
    let Some(FieldValue::Text(before_text)) = before
        .as_ref()
        .filter(|b| !b.deleted)
        .and_then(|b| b.fields.get("text"))
    else {
        return Ok(None);
    };
    if before_text == &text {
        return Ok(None);
    }
    let mut facts = Vec::new();
    for dependency in dependencies {
        if dependency.kind != SemanticDependencyKind::UsesFact {
            continue;
        }
        let SemanticDependencyEndpoint::BibleField {
            node_id,
            part_key,
            field_key,
            field_id: Some(field_id),
        } = &dependency.target
        else {
            continue;
        };
        let Some(binding) = &dependency.revision_binding else {
            continue;
        };
        if binding.source_revision_event_id != generation {
            continue;
        }
        let Some(revision_event_id) =
            crate::bible_field_lineage::current_revision(conn, &dependency.target)?
        else {
            continue;
        };
        let Some(detail) = crate::bible_graph_store::load_node_detail_projection(conn, node_id)?
        else {
            continue;
        };
        let resolved = crate::ai_temporal_context::resolve_fields(
            crate::ai_context_projection::context_fields(detail.parts),
            detail.snapshots,
            None,
        )?;
        let Some(FieldValue::Text(current)) = resolved
            .fields
            .iter()
            .find(|f| f.part_key == *part_key && f.field_key == *field_key)
            .map(|f| &f.value)
        else {
            continue;
        };
        let consumed = crate::revision_projection::load_object_field_projection_at_event(
            conn,
            ObjectKind::BiblePartField,
            field_id.as_str(),
            binding.target_revision_event_id,
        )?;
        let Some(FieldValue::Text(consumed_text)) = consumed
            .as_ref()
            .filter(|h| !h.deleted)
            .and_then(|h| h.fields.get("value"))
        else {
            continue;
        };
        facts.push(ScriptFactField {
            dependency_id: dependency.id.clone(),
            node_id: node_id.clone(),
            part_key: part_key.clone(),
            field_key: field_key.clone(),
            field_id: field_id.clone(),
            consumed_revision_event_id: binding.target_revision_event_id,
            consumed_text: consumed_text.clone(),
            revision_event_id,
            text: current.clone(),
        });
    }
    facts.sort_by(|a, b| a.dependency_id.as_str().cmp(b.dependency_id.as_str()));
    if facts.is_empty() {
        return Ok(None);
    }
    Ok(Some(ScriptFactEditEvidence {
        document_id: ScriptDocumentId::new(document)
            .map_err(|e| HistoryStoreError::InvalidValue(e.to_string()))?,
        segment_id: segment.clone(),
        block_id: edit.block_id,
        before_revision_event_id: edit.expected_revision_event_id,
        revision_event_id: parse_event(&revision)?,
        segment_revision_event_id: parse_event(&segment_revision)?,
        before_text: before_text.clone(),
        text,
        start_ms,
        end_ms,
        generation_event_id: generation,
        facts,
    }))
}

pub(crate) fn capture(
    conn: &Connection,
    request: &RequestScriptFactProposalCommand,
) -> Result<ScriptFactProposalBinding, HistoryStoreError> {
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let result = capture(&tx, request)?;
        tx.commit()?;
        return Ok(result);
    }
    let (generation, block): (String,String) = conn.query_row("SELECT event_id,block_id FROM script_generations WHERE segment_id=?1 ORDER BY rowid DESC LIMIT 1", [request.segment_id.as_str()], |r| Ok((r.get(0)?,r.get(1)?))).optional()?.ok_or_else(stale)?;
    if generation != request.generation_event_id.0.to_string() || block != request.block_id.as_str()
    {
        return Err(stale());
    }
    let dependencies = dependencies(conn, &request.segment_id, request.generation_event_id)?;
    let mut edit = evidence(
        conn,
        &request.segment_id,
        request.generation_event_id,
        &block,
        &dependencies,
    )?
    .ok_or_else(stale)?;
    if edit.document_id != request.document_id
        || edit.revision_event_id != request.expected_block_revision_event_id
    {
        return Err(stale());
    }
    edit.facts.retain(|f| {
        f.dependency_id == request.dependency_id
            && f.revision_event_id == request.expected_field_revision_event_id
    });
    if edit.facts.len() != 1 {
        return Err(stale());
    }
    // Preserve consumed association/name custody; never infer replacement paths.
    let mut context_dependencies = Vec::new();
    for dependency in dependencies {
        let current = match &dependency.target {
            SemanticDependencyEndpoint::BibleEdge { .. } => {
                crate::bible_relationship_lineage::current_revision(conn, &dependency.target)?
            }
            SemanticDependencyEndpoint::BibleNode { .. } => {
                crate::bible_node_name_lineage::current_revision(conn, &dependency.target)?
            }
            _ => continue,
        };
        if current
            != dependency
                .revision_binding
                .as_ref()
                .map(|b| b.target_revision_event_id)
        {
            return Err(stale());
        }
        context_dependencies.push(dependency);
    }
    context_dependencies.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(ScriptFactProposalBinding {
        request: request.clone(),
        edit,
        context_dependencies,
    })
}

pub(crate) fn validate(
    conn: &Connection,
    binding: &ScriptFactProposalBinding,
) -> Result<(), HistoryStoreError> {
    if capture(conn, &binding.request)? != *binding {
        return Err(stale());
    }
    Ok(())
}
pub(crate) fn stale() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Saved edit, consumed fact or relationship changed; analyze the saved edit again.".into(),
    )
}
fn parse_event(value: &str) -> Result<ChangeEventId, HistoryStoreError> {
    uuid::Uuid::parse_str(value)
        .map(ChangeEventId)
        .map_err(|e| HistoryStoreError::InvalidId(e.to_string()))
}
