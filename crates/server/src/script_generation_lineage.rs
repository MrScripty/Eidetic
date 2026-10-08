use std::collections::BTreeMap;

use eidetic_core::contracts::{
    ChangeEventId, GenerateScriptBlockCommand, SemanticDependency, SemanticDependencyEndpoint,
    SemanticDependencyId, SemanticDependencyKind, SemanticDependencyRevisionBinding,
};
use rusqlite::{Connection, Transaction, params};

use crate::history_store::HistoryStoreError;
use crate::semantic_dependency_store;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    crate::story_arc_store::create_schema(conn)?;
    crate::bible_graph_store::create_schema(conn)?;
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
    conn: &Connection,
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
    let mut dependencies = inputs
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
        .collect::<Result<Vec<_>, HistoryStoreError>>()?;
    if let Some(target) = &command.target_binding {
        let input = crate::timeline_notes_lineage::from_target(conn, target)?;
        if let Some(dependency) = crate::timeline_notes_lineage::dependency(
            &input,
            &command.block.segment_id,
            event,
            created_at_ms,
        ) {
            dependencies.push(dependency);
        }
    }
    dependencies.extend(crate::ancestor_notes_lineage::dependencies(
        conn,
        command.ancestor_notes_inputs.as_deref(),
        &command.block.segment_id,
        event,
        created_at_ms,
    )?);
    let mut arc_seen = std::collections::BTreeSet::new();
    for input in command.arc_inputs.iter().flatten() {
        if !arc_seen.insert((input.arc_id.0.to_string(), input.field)) {
            return Err(HistoryStoreError::InvalidValue(
                "duplicate story arc input".into(),
            ));
        }
        let Some(revision) = input.revision_event_id else {
            continue;
        };
        dependencies.push(SemanticDependency {
            id: SemanticDependencyId::new(format!(
                "generation.{}.arc.{}.{}",
                event.0,
                input.arc_id.0,
                input.field.as_str()
            ))
            .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?,
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: command.block.segment_id.clone(),
            },
            target: crate::story_arc_lineage::endpoint(input),
            kind: SemanticDependencyKind::DerivesFrom,
            rationale: Some("Tagged story arc field supplied to generation".into()),
            confidence: None,
            created_at_ms,
            revision_binding: Some(SemanticDependencyRevisionBinding {
                source_revision_event_id: event,
                target_revision_event_id: revision,
            }),
        });
    }
    let mut seen = std::collections::BTreeSet::new();
    for input in command.bible_inputs.iter().flatten() {
        if !seen.insert(input.field_id.as_str()) {
            return Err(HistoryStoreError::InvalidValue(
                "duplicate Bible input".into(),
            ));
        }
        dependencies.push(SemanticDependency {
            id: SemanticDependencyId::new(format!(
                "generation.{}.bible.{}",
                event.0,
                dependencies.len()
            ))
            .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?,
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: command.block.segment_id.clone(),
            },
            target: crate::bible_field_lineage::endpoint(input),
            kind: SemanticDependencyKind::UsesFact,
            rationale: Some("Untimed Bible field supplied to generation".into()),
            confidence: None,
            created_at_ms,
            revision_binding: Some(SemanticDependencyRevisionBinding {
                source_revision_event_id: event,
                target_revision_event_id: input.revision_event_id,
            }),
        });
    }
    let mut seen_names = std::collections::BTreeSet::new();
    for input in command.bible_node_name_inputs.iter().flatten() {
        if !seen_names.insert(input.node_id.as_str()) {
            return Err(HistoryStoreError::InvalidValue(
                "duplicate Bible name input".into(),
            ));
        }
        dependencies.push(SemanticDependency {
            id: SemanticDependencyId::new(format!(
                "generation.{}.name.{}",
                event.0,
                dependencies.len()
            ))
            .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?,
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: command.block.segment_id.clone(),
            },
            target: crate::bible_node_name_lineage::endpoint(input),
            kind: SemanticDependencyKind::UsesFact,
            rationale: Some("Authored Bible name supplied to generation".into()),
            confidence: None,
            created_at_ms,
            revision_binding: Some(SemanticDependencyRevisionBinding {
                source_revision_event_id: event,
                target_revision_event_id: input.revision_event_id,
            }),
        });
    }
    let mut seen_relationships = std::collections::BTreeSet::new();
    for input in command.bible_relationship_inputs.iter().flatten() {
        if !seen_relationships.insert(input.edge.edge_id.as_str()) {
            return Err(HistoryStoreError::InvalidValue(
                "duplicate Bible relationship input".into(),
            ));
        }
        dependencies.push(SemanticDependency {
            id: SemanticDependencyId::new(format!(
                "generation.{}.relationship.{}",
                event.0,
                dependencies.len()
            ))
            .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?,
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: command.block.segment_id.clone(),
            },
            target: crate::bible_relationship_lineage::endpoint(input),
            kind: SemanticDependencyKind::UsesFact,
            rationale: Some("Bible relationship supplied to generation".into()),
            confidence: None,
            created_at_ms,
            revision_binding: Some(SemanticDependencyRevisionBinding {
                source_revision_event_id: event,
                target_revision_event_id: input.revision_event_id,
            }),
        });
    }
    if let Some(dependency) = crate::script_context_scope::dependency(command, event, created_at_ms)
    {
        dependencies.push(dependency);
    }
    if let Some(dependency) = crate::bible_context_scope::dependency(command, event, created_at_ms)
    {
        dependencies.push(dependency);
    }
    Ok(dependencies)
}

pub(crate) fn record_in_transaction(
    tx: &Transaction<'_>,
    command: &GenerateScriptBlockCommand,
    event: ChangeEventId,
    dependencies: &[SemanticDependency],
) -> Result<(), HistoryStoreError> {
    crate::script_context_scope::validate(tx, command)?;
    crate::bible_context_scope::validate(tx, command)?;
    crate::ancestor_notes_lineage::validate_inputs(
        tx,
        command.ancestor_notes_inputs.as_deref().unwrap_or_default(),
    )?;
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
        )",
            params![
                input.block_id.as_str(),
                input.revision_event_id.0.to_string(),
                input.text,
                input.segment_id.as_str(),
            ],
            |row| row.get(0),
        )?;
        if !valid || !historical_segment_matches(tx, input)? {
            return Err(HistoryStoreError::InvalidValue(
                "generation input does not match canonical revision history".into(),
            ));
        }
    }
    for input in command.arc_inputs.iter().flatten() {
        crate::story_arc_lineage::validate_history(tx, input)?;
    }
    for input in command.bible_inputs.iter().flatten() {
        crate::bible_field_lineage::validate_history(tx, input)?;
    }
    for input in command.bible_node_name_inputs.iter().flatten() {
        crate::bible_node_name_lineage::validate_history(tx, input)?;
    }
    for input in command.bible_relationship_inputs.iter().flatten() {
        crate::bible_relationship_lineage::validate_history(tx, input)?;
    }
    tx.execute("INSERT INTO script_generations (event_id, segment_id, block_id, inputs_known) VALUES (?1, ?2, ?3, ?4)",
        params![event.0.to_string(), command.block.segment_id.as_str(), command.block.block_id.as_str(), command.script_inputs.is_some()])?;
    for dependency in dependencies {
        semantic_dependency_store::insert_dependency_in_transaction(tx, dependency, event)?;
    }
    Ok(())
}

fn historical_segment_matches(
    conn: &Connection,
    input: &eidetic_core::contracts::ScriptContextBlock,
) -> Result<bool, HistoryStoreError> {
    use eidetic_core::contracts::{FieldValue, ObjectKind};
    let Some(projection) = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::ScriptSegment,
        input.segment_id.as_str(),
        input.segment_revision_event_id,
    )?
    else {
        return Ok(false);
    };
    let fields = &projection.fields;
    Ok(!projection.deleted
        && fields.get("document_id")
            == Some(&FieldValue::ObjectRef {
                kind: ObjectKind::ScriptDocument,
                id: input.document_id.as_str().to_owned(),
            })
        && fields.get("source_node_id")
            == input
                .source_node_id
                .as_ref()
                .map(|id| FieldValue::ObjectRef {
                    kind: ObjectKind::TimelineNode,
                    id: id.clone(),
                })
                .as_ref()
        && fields.get("start_ms")
            == Some(&FieldValue::Integer(
                i64::try_from(input.start_ms).map_err(|_| {
                    HistoryStoreError::InvalidValue("start_ms exceeds storage range".into())
                })?,
            ))
        && fields.get("end_ms")
            == Some(&FieldValue::Integer(i64::try_from(input.end_ms).map_err(
                |_| HistoryStoreError::InvalidValue("end_ms exceeds storage range".into()),
            )?)))
}

#[cfg(test)]
#[path = "script_generation_lineage_tests.rs"]
mod tests;
