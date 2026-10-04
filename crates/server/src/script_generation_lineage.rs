use std::collections::BTreeMap;

use eidetic_core::contracts::{
    ChangeEventId, GenerateScriptBlockCommand, SemanticDependency, SemanticDependencyEndpoint,
    SemanticDependencyId, SemanticDependencyKind, SemanticDependencyRevisionBinding,
};
use rusqlite::{Connection, Transaction, params};

use crate::history_store::HistoryStoreError;
use crate::semantic_dependency_store;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    semantic_dependency_store::create_schema(conn)
        .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS script_generations (
        event_id TEXT PRIMARY KEY REFERENCES change_events(id),
        segment_id TEXT NOT NULL REFERENCES script_segments(id),
        block_id TEXT NOT NULL REFERENCES script_blocks(id),
        inputs_known INTEGER NOT NULL CHECK (inputs_known IN (0, 1))
    ); CREATE INDEX IF NOT EXISTS idx_script_generations_segment ON script_generations(segment_id);")?;
    Ok(())
}

pub(crate) fn dependencies(
    command: &GenerateScriptBlockCommand,
    event: ChangeEventId,
    created_at_ms: u64,
) -> Result<Vec<SemanticDependency>, HistoryStoreError> {
    let mut inputs = BTreeMap::new();
    for input in command.script_inputs.iter().flatten() {
        let block_previous = inputs.insert(
            ("script_block", input.block_id.as_str().to_owned()),
            (
                SemanticDependencyEndpoint::ScriptBlock {
                    block_id: input.block_id.clone(),
                },
                input.revision_event_id,
            ),
        );
        let segment_previous = inputs.insert(
            ("script_segment", input.segment_id.as_str().to_owned()),
            (
                SemanticDependencyEndpoint::ScriptSegment {
                    segment_id: input.segment_id.clone(),
                },
                input.segment_revision_event_id,
            ),
        );
        if block_previous.is_some_and(|(_, revision)| revision != input.revision_event_id)
            || segment_previous
                .is_some_and(|(_, revision)| revision != input.segment_revision_event_id)
        {
            return Err(HistoryStoreError::InvalidValue(
                "generation input contains conflicting revision bindings".into(),
            ));
        }
    }
    inputs
        .into_iter()
        .enumerate()
        .map(|(index, (_, (target, revision)))| {
            Ok(SemanticDependency {
                id: SemanticDependencyId::new(format!("generation.{}.input.{index}", event.0))
                    .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?,
                source: SemanticDependencyEndpoint::ScriptSegment {
                    segment_id: command.block.segment_id.clone(),
                },
                target,
                kind: SemanticDependencyKind::DerivesFrom,
                rationale: Some("Screenplay evidence supplied to generation".into()),
                confidence: None,
                created_at_ms,
                revision_binding: Some(SemanticDependencyRevisionBinding {
                    source_revision_event_id: event,
                    target_revision_event_id: revision,
                }),
            })
        })
        .collect()
}

pub(crate) fn record_in_transaction(
    tx: &Transaction<'_>,
    command: &GenerateScriptBlockCommand,
    event: ChangeEventId,
    dependencies: &[SemanticDependency],
) -> Result<(), HistoryStoreError> {
    for input in command.script_inputs.iter().flatten() {
        if input.document_id != command.block.document_id {
            return Err(HistoryStoreError::InvalidValue(
                "generation input belongs to another document".into(),
            ));
        }
        // The captured revision may no longer be current or its source may be
        // deleted. Validate historical evidence, never silently rebind to latest.
        let valid: bool = tx.query_row(
            "SELECT EXISTS (
            SELECT 1 FROM object_revisions r JOIN object_revision_fields f ON f.revision_id = r.id
            WHERE r.object_kind = 'script_block' AND r.object_id = ?1 AND r.change_event_id = ?2
                AND f.field_key = 'text' AND f.new_type = 'text' AND f.new_text = ?3
        ) AND EXISTS (
            SELECT 1 FROM object_revisions r JOIN object_revision_fields f ON f.revision_id = r.id
            WHERE r.object_kind = 'script_block' AND r.object_id = ?1 AND r.change_event_id = ?2
                AND f.field_key = 'segment_id' AND f.new_ref_id = ?4
        ) AND (
            SELECT COUNT(*) FROM object_revisions r JOIN object_revision_fields f ON f.revision_id = r.id
            WHERE r.object_kind = 'script_segment' AND r.object_id = ?4 AND r.change_event_id = ?5
                AND ((f.field_key = 'document_id' AND f.new_ref_id = ?6)
                    OR (f.field_key = 'source_node_id' AND f.new_ref_id IS ?7)
                    OR (f.field_key = 'start_ms' AND f.new_type = 'integer' AND f.new_integer = ?8)
                    OR (f.field_key = 'end_ms' AND f.new_type = 'integer' AND f.new_integer = ?9))
        ) = 4",
            params![
                input.block_id.as_str(),
                input.revision_event_id.0.to_string(),
                input.text,
                input.segment_id.as_str(),
                input.segment_revision_event_id.0.to_string(),
                input.document_id.as_str(),
                input.source_node_id,
                input.start_ms,
                input.end_ms
            ],
            |row| row.get(0),
        )?;
        if !valid {
            return Err(HistoryStoreError::InvalidValue(
                "generation input does not match canonical revision history".into(),
            ));
        }
    }
    tx.execute("INSERT INTO script_generations (event_id, segment_id, block_id, inputs_known) VALUES (?1, ?2, ?3, ?4)",
        params![event.0.to_string(), command.block.segment_id.as_str(), command.block.block_id.as_str(), command.script_inputs.is_some()])?;
    for dependency in dependencies {
        semantic_dependency_store::insert_dependency_in_transaction(tx, dependency, event)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "script_generation_lineage_tests.rs"]
mod tests;
