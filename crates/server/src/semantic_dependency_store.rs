#[cfg(test)]
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectRevision,
    RecordSemanticDependencyCommand, RevisionOperation,
};
use eidetic_core::contracts::{
    ObjectKind, ProjectionEnvelope, ProjectionVersion, SemanticDependency,
    SemanticDependencyEndpoint, SemanticDependencyId, SemanticDependencyProjection,
};
use rusqlite::{Connection, Row, params};
#[cfg(test)]
use rusqlite::{OptionalExtension, Transaction};

#[cfg(test)]
use crate::history_store::RecordChangeOutcome;
use crate::history_store::{self, HistoryStoreError};

const SEMANTIC_DEPENDENCY_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS semantic_dependencies (
    id TEXT PRIMARY KEY CHECK (id <> ''),
    source_kind TEXT NOT NULL,
    source_id TEXT NOT NULL CHECK (source_id <> ''),
    source_part_key TEXT,
    source_field_key TEXT,
    source_field_id TEXT,
    target_kind TEXT NOT NULL,
    target_id TEXT NOT NULL CHECK (target_id <> ''),
    target_part_key TEXT,
    target_field_key TEXT,
    target_field_id TEXT,
    dependency_kind TEXT NOT NULL,
    rationale TEXT,
    confidence REAL,
    created_at_ms INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_semantic_dependencies_source
    ON semantic_dependencies(source_kind, source_id);
CREATE INDEX IF NOT EXISTS idx_semantic_dependencies_target
    ON semantic_dependencies(target_kind, target_id);
CREATE INDEX IF NOT EXISTS idx_semantic_dependencies_target_field
    ON semantic_dependencies(target_kind, target_id, target_part_key, target_field_key);
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SemanticDependencyFilter {
    pub endpoint: DependencyEndpointFilter,
    pub direction: DependencyDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DependencyDirection {
    Source,
    Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DependencyEndpointFilter {
    pub kind: String,
    pub id: String,
    pub part_key: Option<String>,
    pub field_key: Option<String>,
}

pub(crate) fn create_schema(conn: &Connection) -> Result<(), SemanticDependencyStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(SEMANTIC_DEPENDENCY_SCHEMA_SQL)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn record_semantic_dependency(
    conn: &mut Connection,
    command: &CommandEnvelope<RecordSemanticDependencyCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, SemanticDependencyStoreError> {
    create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.dependency_record")?
    {
        return Ok(outcome);
    }
    validate_dependency(&command.payload.dependency)?;
    if dependency_exists(conn, &command.payload.dependency.id)? {
        return Err(SemanticDependencyStoreError::InvalidCommand(format!(
            "semantic dependency already exists: {}",
            command.payload.dependency.id.as_str()
        )));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!(
            "record semantic dependency {}",
            command.payload.dependency.id.as_str()
        ),
    )
    .with_created_at_ms(created_at_ms);
    let revision = dependency_revision(&command.payload.dependency, event.id)?;

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.dependency_record",
        &event,
        &[revision],
        |tx| insert_dependency_in_transaction(tx, &command.payload.dependency, event.id),
    )?)
}

pub(crate) fn load_semantic_dependency_projection(
    conn: &Connection,
    filter: &SemanticDependencyFilter,
) -> Result<ProjectionEnvelope<SemanticDependencyProjection>, SemanticDependencyStoreError> {
    create_schema(conn)?;
    let dependencies = load_dependencies(conn, filter)?;
    let summary =
        history_store::load_revision_summary_for_kind(conn, ObjectKind::SemanticDependency)?;
    let projection = SemanticDependencyProjection { dependencies };

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}

#[cfg(test)]
fn validate_dependency(
    dependency: &SemanticDependency,
) -> Result<(), SemanticDependencyStoreError> {
    if let Some(confidence) = dependency.confidence {
        if !(0.0..=1.0).contains(&confidence) {
            return Err(SemanticDependencyStoreError::InvalidCommand(
                "confidence must be between 0 and 1".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
fn dependency_exists(
    conn: &Connection,
    dependency_id: &SemanticDependencyId,
) -> Result<bool, SemanticDependencyStoreError> {
    conn.query_row(
        "SELECT 1 FROM semantic_dependencies WHERE id = ?1 AND deleted_event_id IS NULL",
        [dependency_id.as_str()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(SemanticDependencyStoreError::from)
}

#[cfg(test)]
fn insert_dependency_in_transaction(
    tx: &Transaction<'_>,
    dependency: &SemanticDependency,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let source = SqlEndpoint::from_endpoint(&dependency.source);
    let target = SqlEndpoint::from_endpoint(&dependency.target);
    tx.execute(
        "INSERT INTO semantic_dependencies (
            id,
            source_kind, source_id, source_part_key, source_field_key, source_field_id,
            target_kind, target_id, target_part_key, target_field_key, target_field_id,
            dependency_kind, rationale, confidence, created_at_ms, created_event_id
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
         )",
        params![
            dependency.id.as_str(),
            source.kind,
            source.id,
            source.part_key,
            source.field_key,
            source.field_id,
            target.kind,
            target.id,
            target.part_key,
            target.field_key,
            target.field_id,
            encode_string_enum(&dependency.kind)?,
            dependency.rationale,
            dependency.confidence,
            dependency.created_at_ms as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn load_dependencies(
    conn: &Connection,
    filter: &SemanticDependencyFilter,
) -> Result<Vec<SemanticDependency>, SemanticDependencyStoreError> {
    let prefix = match filter.direction {
        DependencyDirection::Source => "source",
        DependencyDirection::Target => "target",
    };
    let sql = format!(
        "SELECT
            id,
            source_kind, source_id, source_part_key, source_field_key, source_field_id,
            target_kind, target_id, target_part_key, target_field_key, target_field_id,
            dependency_kind, rationale, confidence, created_at_ms
         FROM semantic_dependencies
         WHERE deleted_event_id IS NULL
           AND {prefix}_kind = ?1
           AND {prefix}_id = ?2
           AND (?3 IS NULL OR {prefix}_part_key = ?3)
           AND (?4 IS NULL OR {prefix}_field_key = ?4)
         ORDER BY created_at_ms ASC, id ASC"
    );

    let mut statement = conn.prepare(&sql)?;
    let mut rows = statement.query(params![
        filter.endpoint.kind,
        filter.endpoint.id,
        filter.endpoint.part_key,
        filter.endpoint.field_key,
    ])?;
    let mut dependencies = Vec::new();
    while let Some(row) = rows.next()? {
        dependencies.push(row_to_dependency(row)?);
    }
    Ok(dependencies)
}

fn row_to_dependency(row: &Row<'_>) -> Result<SemanticDependency, SemanticDependencyStoreError> {
    let id: String = row.get(0)?;
    let dependency_kind: String = row.get(11)?;
    let created_at_ms: i64 = row.get(14)?;
    Ok(SemanticDependency {
        id: SemanticDependencyId::new(id)?,
        source: SqlEndpoint {
            kind: row.get(1)?,
            id: row.get(2)?,
            part_key: row.get(3)?,
            field_key: row.get(4)?,
            field_id: row.get(5)?,
        }
        .into_endpoint()?,
        target: SqlEndpoint {
            kind: row.get(6)?,
            id: row.get(7)?,
            part_key: row.get(8)?,
            field_key: row.get(9)?,
            field_id: row.get(10)?,
        }
        .into_endpoint()?,
        kind: decode_string_enum(&dependency_kind)?,
        rationale: row.get(12)?,
        confidence: row.get(13)?,
        created_at_ms: u64::try_from(created_at_ms).map_err(|e| {
            SemanticDependencyStoreError::InvalidCommand(format!(
                "invalid created_at_ms for semantic dependency: {e}"
            ))
        })?,
    })
}

#[cfg(test)]
fn dependency_revision(
    dependency: &SemanticDependency,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    let revision = ObjectRevision::new(
        ObjectKind::SemanticDependency,
        dependency.id.as_str(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "dependency_kind",
        None,
        Some(FieldValue::Text(encode_string_enum(&dependency.kind)?)),
    ))
    .with_field(FieldDelta::new(
        "source",
        None,
        Some(FieldValue::Text(endpoint_label(&dependency.source))),
    ))
    .with_field(FieldDelta::new(
        "target",
        None,
        Some(FieldValue::Text(endpoint_label(&dependency.target))),
    ));
    let revision = match dependency.rationale.as_ref() {
        Some(rationale) => revision.with_field(FieldDelta::new(
            "rationale",
            None,
            Some(FieldValue::Text(rationale.clone())),
        )),
        None => revision,
    };
    Ok(match dependency.confidence {
        Some(confidence) => revision.with_field(FieldDelta::new(
            "confidence",
            None,
            Some(FieldValue::Number(f64::from(confidence))),
        )),
        None => revision,
    })
}

#[cfg(test)]
fn endpoint_label(endpoint: &SemanticDependencyEndpoint) -> String {
    let sql = SqlEndpoint::from_endpoint(endpoint);
    match (sql.part_key, sql.field_key) {
        (Some(part_key), Some(field_key)) => {
            format!("{}:{}.{part_key}.{field_key}", sql.kind, sql.id)
        }
        _ => format!("{}:{}", sql.kind, sql.id),
    }
}

#[derive(Debug)]
struct SqlEndpoint {
    kind: String,
    id: String,
    part_key: Option<String>,
    field_key: Option<String>,
    field_id: Option<String>,
}

impl SqlEndpoint {
    #[cfg(test)]
    fn from_endpoint(endpoint: &SemanticDependencyEndpoint) -> Self {
        match endpoint {
            SemanticDependencyEndpoint::TimelineNode { node_id } => Self {
                kind: "timeline_node".to_string(),
                id: node_id.0.to_string(),
                part_key: None,
                field_key: None,
                field_id: None,
            },
            SemanticDependencyEndpoint::BibleNode { node_id } => Self {
                kind: "bible_node".to_string(),
                id: node_id.as_str().to_string(),
                part_key: None,
                field_key: None,
                field_id: None,
            },
            SemanticDependencyEndpoint::BibleField {
                node_id,
                part_key,
                field_key,
                field_id,
            } => Self {
                kind: "bible_field".to_string(),
                id: node_id.as_str().to_string(),
                part_key: Some(part_key.as_str().to_string()),
                field_key: Some(field_key.as_str().to_string()),
                field_id: field_id.as_ref().map(|id| id.as_str().to_string()),
            },
            SemanticDependencyEndpoint::ScriptSegment { segment_id } => Self {
                kind: "script_segment".to_string(),
                id: segment_id.as_str().to_string(),
                part_key: None,
                field_key: None,
                field_id: None,
            },
            SemanticDependencyEndpoint::ScriptBlock { block_id } => Self {
                kind: "script_block".to_string(),
                id: block_id.as_str().to_string(),
                part_key: None,
                field_key: None,
                field_id: None,
            },
        }
    }

    fn into_endpoint(self) -> Result<SemanticDependencyEndpoint, SemanticDependencyStoreError> {
        match self.kind.as_str() {
            "timeline_node" => Ok(SemanticDependencyEndpoint::TimelineNode {
                node_id: eidetic_core::timeline::node::NodeId(parse_uuid(&self.id)?),
            }),
            "bible_node" => Ok(SemanticDependencyEndpoint::BibleNode {
                node_id: eidetic_core::contracts::BibleGraphNodeId::new(self.id)?,
            }),
            "bible_field" => Ok(SemanticDependencyEndpoint::BibleField {
                node_id: eidetic_core::contracts::BibleGraphNodeId::new(self.id)?,
                part_key: eidetic_core::contracts::BibleGraphPartKey::new(required(
                    self.part_key,
                    "part_key",
                )?)?,
                field_key: eidetic_core::contracts::BibleGraphFieldKey::new(required(
                    self.field_key,
                    "field_key",
                )?)?,
                field_id: self
                    .field_id
                    .map(eidetic_core::contracts::BibleGraphFieldId::new)
                    .transpose()?,
            }),
            "script_segment" => Ok(SemanticDependencyEndpoint::ScriptSegment {
                segment_id: eidetic_core::contracts::ScriptSegmentId::new(self.id)?,
            }),
            "script_block" => Ok(SemanticDependencyEndpoint::ScriptBlock {
                block_id: eidetic_core::contracts::ScriptBlockId::new(self.id)?,
            }),
            other => Err(SemanticDependencyStoreError::InvalidCommand(format!(
                "unknown semantic dependency endpoint kind: {other}"
            ))),
        }
    }
}

#[cfg(test)]
fn encode_string_enum<T>(value: &T) -> Result<String, HistoryStoreError>
where
    T: serde::Serialize,
{
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected enum to serialize as string".to_string(),
        )),
    }
}

fn decode_string_enum<T>(value: &str) -> Result<T, SemanticDependencyStoreError>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_json::from_value(serde_json::Value::String(
        value.to_string(),
    ))?)
}

fn parse_uuid(value: &str) -> Result<uuid::Uuid, SemanticDependencyStoreError> {
    uuid::Uuid::parse_str(value)
        .map_err(|e| SemanticDependencyStoreError::InvalidCommand(e.to_string()))
}

fn required<T>(
    value: Option<T>,
    field_name: &'static str,
) -> Result<T, SemanticDependencyStoreError> {
    value.ok_or_else(|| {
        SemanticDependencyStoreError::InvalidCommand(format!(
            "missing semantic dependency endpoint {field_name}"
        ))
    })
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SemanticDependencyStoreError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Contract(#[from] eidetic_core::contracts::SemanticDependencyContractError),
    #[error(transparent)]
    BibleGraphContract(#[from] eidetic_core::contracts::BibleGraphContractError),
    #[error(transparent)]
    ScriptContract(#[from] eidetic_core::contracts::ScriptContractError),
}

#[cfg(test)]
#[path = "semantic_dependency_store_tests.rs"]
mod tests;
