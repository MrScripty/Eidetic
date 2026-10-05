use eidetic_core::contracts::{
    ChangeEventId, ScriptImpactCause, ScriptImpactProjection, ScriptImpactReason, ScriptSegmentId,
    SemanticDependencyEndpoint,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::history_store::HistoryStoreError;
use crate::semantic_dependency_store::{
    self, DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
};

pub(crate) fn load_impact(
    conn: &Connection,
    segment: &ScriptSegmentId,
) -> Result<Option<ScriptImpactProjection>, HistoryStoreError> {
    let generation = conn
        .query_row(
            "SELECT event_id, inputs_known, block_id FROM script_generations
        WHERE segment_id = ?1 ORDER BY rowid DESC LIMIT 1",
            [segment.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?;
    let Some((event, lineage_available, block)) = generation else {
        return Ok(None);
    };
    let generation_event_id = parse_event(&event)?;
    let dependencies = semantic_dependency_store::load_semantic_dependency_projection(
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
    .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?
    .payload
    .dependencies;
    let mut causes = Vec::new();
    for dependency in dependencies {
        let Some(ref binding) = dependency.revision_binding else {
            continue;
        };
        if binding.source_revision_event_id != generation_event_id {
            continue;
        }
        if matches!(
            dependency.target,
            SemanticDependencyEndpoint::TimelineNode { .. }
        ) {
            if let Some(cause) =
                crate::script_context_scope::cause(conn, generation_event_id, segment, &dependency)?
            {
                causes.push(cause);
            }
            continue;
        }
        let state = match &dependency.target {
            SemanticDependencyEndpoint::ScriptBlock { block_id } => conn.query_row(
                "SELECT b.updated_event_id, b.deleted_event_id IS NOT NULL OR s.deleted_event_id IS NOT NULL OR d.deleted_event_id IS NOT NULL, s.id
                 FROM script_blocks b JOIN script_segments s ON s.id = b.segment_id
                 JOIN script_documents d ON d.id = s.document_id WHERE b.id = ?1", [block_id.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?, row.get::<_, String>(2)?))).optional()?,
            SemanticDependencyEndpoint::ScriptSegment { segment_id } => conn.query_row(
                "SELECT s.updated_event_id, s.deleted_event_id IS NOT NULL OR d.deleted_event_id IS NOT NULL, s.id
                 FROM script_segments s JOIN script_documents d ON d.id = s.document_id WHERE s.id = ?1", [segment_id.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?, row.get::<_, String>(2)?))).optional()?,
            SemanticDependencyEndpoint::BibleField { field_id: Some(_), .. } =>
                crate::bible_field_lineage::current_revision(conn, &dependency.target)?
                    .map(|revision| (revision.0.to_string(), false, String::new())),
            _ => continue,
        };
        // A generation can read its own previous draft. Retain that audit input
        // but do not flag its intentional replacement as an external impact.
        if state
            .as_ref()
            .is_some_and(|(_, _, input_segment)| input_segment == segment.as_str())
        {
            continue;
        }
        let (current_revision_event_id, reason) = match state {
            Some((revision, false, _)) => {
                let current = parse_event(&revision)?;
                if current == binding.target_revision_event_id {
                    continue;
                }
                (Some(current), ScriptImpactReason::Changed)
            }
            _ => (None, ScriptImpactReason::Deleted),
        };
        let input_excerpt = match &dependency.target {
            SemanticDependencyEndpoint::ScriptBlock { block_id } => conn.query_row(
                "SELECT f.new_text FROM object_revisions r JOIN object_revision_fields f ON f.revision_id = r.id
                 WHERE r.object_kind = 'script_block' AND r.object_id = ?1 AND r.change_event_id = ?2 AND f.field_key = 'text'",
                params![block_id.as_str(), binding.target_revision_event_id.0.to_string()], |row| row.get::<_, Option<String>>(0)).optional()?.flatten()
                .map(|text| text.chars().take(120).collect()),
            SemanticDependencyEndpoint::BibleField { .. } => crate::bible_field_lineage::excerpt(
                conn, &dependency.target, binding.target_revision_event_id)?,
            _ => None,
        };
        causes.push(ScriptImpactCause {
            dependency_id: dependency.id,
            input: dependency.target,
            consumed_revision_event_id: binding.target_revision_event_id,
            current_revision_event_id,
            reason,
            input_excerpt,
        });
    }
    if let Some(cause) = crate::bible_context_scope::cause(conn, generation_event_id, segment)? {
        causes.push(cause);
    }
    Ok(Some(ScriptImpactProjection {
        generation_event_id,
        output_block_id: Some(
            eidetic_core::contracts::ScriptBlockId::new(block)
                .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?,
        ),
        lineage_available,
        needs_review: !causes.is_empty(),
        causes,
    }))
}

fn parse_event(value: &str) -> Result<ChangeEventId, HistoryStoreError> {
    uuid::Uuid::parse_str(value)
        .map(ChangeEventId)
        .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
}
