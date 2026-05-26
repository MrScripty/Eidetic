use eidetic_core::contracts::{
    AgentRun, AgentRunId, AgentToolCall, AgentToolCallId, AgentToolResult, ChangeEvent,
    ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectKind, ObjectRevision,
    RevisionOperation,
};
#[cfg(test)]
use eidetic_core::contracts::{AgentRunStatus, AgentToolCallStatus, AgentToolResultStatus};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

const AGENT_WORKFLOW_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS agent_runs (
    id                TEXT PRIMARY KEY CHECK (id <> ''),
    workflow_id       TEXT NOT NULL CHECK (workflow_id <> ''),
    intent            TEXT NOT NULL CHECK (intent <> ''),
    status            TEXT NOT NULL CHECK (status <> ''),
    created_at_ms     INTEGER NOT NULL,
    completed_at_ms   INTEGER,
    error             TEXT,
    created_event_id  TEXT NOT NULL REFERENCES change_events(id),
    updated_event_id  TEXT NOT NULL REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_agent_runs_workflow
    ON agent_runs(workflow_id, created_at_ms, id);
CREATE INDEX IF NOT EXISTS idx_agent_runs_status
    ON agent_runs(status, created_at_ms, id);

CREATE TABLE IF NOT EXISTS agent_tool_calls (
    id                TEXT PRIMARY KEY CHECK (id <> ''),
    run_id            TEXT NOT NULL REFERENCES agent_runs(id),
    sequence          INTEGER NOT NULL,
    tool_name         TEXT NOT NULL CHECK (tool_name <> ''),
    tool_kind         TEXT NOT NULL CHECK (tool_kind <> ''),
    arguments_json    TEXT NOT NULL,
    status            TEXT NOT NULL CHECK (status <> ''),
    created_at_ms     INTEGER NOT NULL,
    created_event_id  TEXT NOT NULL REFERENCES change_events(id),
    updated_event_id  TEXT NOT NULL REFERENCES change_events(id),
    UNIQUE (run_id, sequence)
);
CREATE INDEX IF NOT EXISTS idx_agent_tool_calls_run
    ON agent_tool_calls(run_id, sequence, id);
CREATE INDEX IF NOT EXISTS idx_agent_tool_calls_tool
    ON agent_tool_calls(tool_name, created_at_ms, id);

CREATE TABLE IF NOT EXISTS agent_tool_results (
    call_id           TEXT PRIMARY KEY REFERENCES agent_tool_calls(id),
    status            TEXT NOT NULL CHECK (status <> ''),
    payload_json      TEXT NOT NULL,
    completed_at_ms   INTEGER NOT NULL,
    created_event_id  TEXT NOT NULL REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_agent_tool_results_status
    ON agent_tool_results(status, completed_at_ms, call_id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(AGENT_WORKFLOW_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn record_agent_run(
    conn: &mut Connection,
    command: &CommandEnvelope<AgentRun>,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "agent.run")? {
        return Ok(outcome);
    }
    validate_agent_run(&command.payload)?;

    let existing = load_agent_run(conn, command.payload.id)?;
    let operation = if existing.is_some() {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AgentWorkflow,
        format!("record agent run {}", command.payload.id.0),
    )
    .with_created_at_ms(command.payload.created_at_ms);
    let revision = agent_run_revision(existing.as_ref(), &command.payload, event.id, &operation)?;

    history_store::record_change_with(conn, command, "agent.run", &event, &[revision], |tx| {
        upsert_agent_run_in_transaction(tx, &command.payload, event.id, operation)
    })
}

pub(crate) fn record_agent_tool_call(
    conn: &mut Connection,
    command: &CommandEnvelope<AgentToolCall>,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "agent.tool_call")?
    {
        return Ok(outcome);
    }
    validate_agent_tool_call(conn, &command.payload)?;

    let existing = load_agent_tool_call(conn, command.payload.id)?;
    let operation = if existing.is_some() {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AgentWorkflow,
        format!("record agent tool call {}", command.payload.id.0),
    )
    .with_created_at_ms(command.payload.created_at_ms);
    let revision =
        agent_tool_call_revision(existing.as_ref(), &command.payload, event.id, &operation)?;

    history_store::record_change_with(
        conn,
        command,
        "agent.tool_call",
        &event,
        &[revision],
        |tx| upsert_agent_tool_call_in_transaction(tx, &command.payload, event.id, operation),
    )
}

pub(crate) fn record_agent_tool_result(
    conn: &mut Connection,
    command: &CommandEnvelope<AgentToolResult>,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "agent.tool_result")?
    {
        return Ok(outcome);
    }
    validate_agent_tool_result(conn, &command.payload)?;

    let existing = load_agent_tool_result(conn, command.payload.call_id)?;
    let operation = if existing.is_some() {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AgentWorkflow,
        format!("record agent tool result {}", command.payload.call_id.0),
    )
    .with_created_at_ms(command.payload.completed_at_ms);
    let revision =
        agent_tool_result_revision(existing.as_ref(), &command.payload, event.id, &operation)?;

    history_store::record_change_with(
        conn,
        command,
        "agent.tool_result",
        &event,
        &[revision],
        |tx| upsert_agent_tool_result_in_transaction(tx, &command.payload, event.id),
    )
}

pub(crate) fn load_agent_run(
    conn: &Connection,
    run_id: AgentRunId,
) -> Result<Option<AgentRun>, HistoryStoreError> {
    create_schema(conn)?;
    conn.query_row(
        "SELECT id, workflow_id, intent, status, created_at_ms, completed_at_ms, error
         FROM agent_runs
         WHERE id = ?1",
        [run_id.0.to_string()],
        row_to_agent_run,
    )
    .optional()
    .map_err(HistoryStoreError::from)
}

pub(crate) fn load_agent_tool_calls(
    conn: &Connection,
    run_id: AgentRunId,
) -> Result<Vec<AgentToolCall>, HistoryStoreError> {
    create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT id, run_id, sequence, tool_name, arguments_json, status, created_at_ms
         FROM agent_tool_calls
         WHERE run_id = ?1
         ORDER BY sequence ASC, id ASC",
    )?;
    let rows = statement.query_map([run_id.0.to_string()], row_to_agent_tool_call)?;
    let mut calls = Vec::new();
    for row in rows {
        calls.push(row?);
    }
    Ok(calls)
}

pub(crate) fn load_agent_tool_result(
    conn: &Connection,
    call_id: AgentToolCallId,
) -> Result<Option<AgentToolResult>, HistoryStoreError> {
    create_schema(conn)?;
    conn.query_row(
        "SELECT call_id, status, payload_json, completed_at_ms
         FROM agent_tool_results
         WHERE call_id = ?1",
        [call_id.0.to_string()],
        row_to_agent_tool_result,
    )
    .optional()
    .map_err(HistoryStoreError::from)
}

fn validate_agent_run(run: &AgentRun) -> Result<(), HistoryStoreError> {
    if run.workflow_id.as_str().trim().is_empty() {
        return Err(HistoryStoreError::InvalidValue(
            "agent run workflow id must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_agent_tool_call(
    conn: &Connection,
    call: &AgentToolCall,
) -> Result<(), HistoryStoreError> {
    if load_agent_run(conn, call.run_id)?.is_none() {
        return Err(HistoryStoreError::InvalidValue(format!(
            "agent tool call references unknown run: {}",
            call.run_id.0
        )));
    }
    Ok(())
}

fn validate_agent_tool_result(
    conn: &Connection,
    result: &AgentToolResult,
) -> Result<(), HistoryStoreError> {
    if load_agent_tool_call(conn, result.call_id)?.is_none() {
        return Err(HistoryStoreError::InvalidValue(format!(
            "agent tool result references unknown call: {}",
            result.call_id.0
        )));
    }
    Ok(())
}

fn upsert_agent_run_in_transaction(
    tx: &Transaction<'_>,
    run: &AgentRun,
    event_id: eidetic_core::contracts::ChangeEventId,
    operation: RevisionOperation,
) -> Result<(), HistoryStoreError> {
    match operation {
        RevisionOperation::Create => {
            tx.execute(
                "INSERT INTO agent_runs (
                    id, workflow_id, intent, status, created_at_ms, completed_at_ms, error,
                    created_event_id, updated_event_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                params![
                    run.id.0.to_string(),
                    run.workflow_id.as_str(),
                    encode_string_enum(&run.intent)?,
                    encode_string_enum(&run.status)?,
                    run.created_at_ms as i64,
                    optional_u64_to_i64(run.completed_at_ms),
                    run.error,
                    event_id.0.to_string(),
                ],
            )?;
        }
        RevisionOperation::Update => {
            tx.execute(
                "UPDATE agent_runs
                 SET status = ?2,
                     completed_at_ms = ?3,
                     error = ?4,
                     updated_event_id = ?5
                 WHERE id = ?1",
                params![
                    run.id.0.to_string(),
                    encode_string_enum(&run.status)?,
                    optional_u64_to_i64(run.completed_at_ms),
                    run.error,
                    event_id.0.to_string(),
                ],
            )?;
        }
        RevisionOperation::Delete => {
            return Err(HistoryStoreError::InvalidValue(
                "agent run storage does not support delete revisions".to_string(),
            ));
        }
    }
    Ok(())
}

fn upsert_agent_tool_call_in_transaction(
    tx: &Transaction<'_>,
    call: &AgentToolCall,
    event_id: eidetic_core::contracts::ChangeEventId,
    operation: RevisionOperation,
) -> Result<(), HistoryStoreError> {
    match operation {
        RevisionOperation::Create => {
            tx.execute(
                "INSERT INTO agent_tool_calls (
                    id, run_id, sequence, tool_name, tool_kind, arguments_json, status,
                    created_at_ms, created_event_id, updated_event_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                params![
                    call.id.0.to_string(),
                    call.run_id.0.to_string(),
                    call.sequence as i64,
                    call.request.tool_name.as_str(),
                    encode_string_enum(&call.request.arguments.kind())?,
                    serde_json::to_string(&call.request.arguments)?,
                    encode_string_enum(&call.status)?,
                    call.created_at_ms as i64,
                    event_id.0.to_string(),
                ],
            )?;
        }
        RevisionOperation::Update => {
            tx.execute(
                "UPDATE agent_tool_calls
                 SET status = ?2,
                     updated_event_id = ?3
                 WHERE id = ?1",
                params![
                    call.id.0.to_string(),
                    encode_string_enum(&call.status)?,
                    event_id.0.to_string(),
                ],
            )?;
        }
        RevisionOperation::Delete => {
            return Err(HistoryStoreError::InvalidValue(
                "agent tool call storage does not support delete revisions".to_string(),
            ));
        }
    }
    Ok(())
}

fn upsert_agent_tool_result_in_transaction(
    tx: &Transaction<'_>,
    result: &AgentToolResult,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO agent_tool_results (
            call_id, status, payload_json, completed_at_ms, created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(call_id) DO UPDATE SET
            status = excluded.status,
            payload_json = excluded.payload_json,
            completed_at_ms = excluded.completed_at_ms,
            created_event_id = excluded.created_event_id",
        params![
            result.call_id.0.to_string(),
            encode_string_enum(&result.status)?,
            serde_json::to_string(&result.payload)?,
            result.completed_at_ms as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn load_agent_tool_call(
    conn: &Connection,
    call_id: AgentToolCallId,
) -> Result<Option<AgentToolCall>, HistoryStoreError> {
    conn.query_row(
        "SELECT id, run_id, sequence, tool_name, arguments_json, status, created_at_ms
         FROM agent_tool_calls
         WHERE id = ?1",
        [call_id.0.to_string()],
        row_to_agent_tool_call,
    )
    .optional()
    .map_err(HistoryStoreError::from)
}

fn agent_run_revision(
    before: Option<&AgentRun>,
    after: &AgentRun,
    event_id: eidetic_core::contracts::ChangeEventId,
    operation: &RevisionOperation,
) -> Result<ObjectRevision, HistoryStoreError> {
    let mut revision = ObjectRevision::new(
        ObjectKind::AgentRun,
        after.id.0.to_string(),
        event_id,
        operation.clone(),
    );
    revision = revision.with_field(FieldDelta::new(
        "status",
        before
            .map(|run| encode_string_enum(&run.status).map(FieldValue::Text))
            .transpose()?,
        Some(FieldValue::Text(encode_string_enum(&after.status)?)),
    ));
    if before.map(|run| &run.error) != Some(&after.error) {
        revision = revision.with_field(FieldDelta::new(
            "error",
            before
                .and_then(|run| run.error.clone())
                .map(FieldValue::Text),
            after.error.clone().map(FieldValue::Text),
        ));
    }
    Ok(revision)
}

fn agent_tool_call_revision(
    before: Option<&AgentToolCall>,
    after: &AgentToolCall,
    event_id: eidetic_core::contracts::ChangeEventId,
    operation: &RevisionOperation,
) -> Result<ObjectRevision, HistoryStoreError> {
    let mut revision = ObjectRevision::new(
        ObjectKind::AgentToolCall,
        after.id.0.to_string(),
        event_id,
        operation.clone(),
    );
    revision = revision.with_field(FieldDelta::new(
        "status",
        before
            .map(|call| encode_string_enum(&call.status).map(FieldValue::Text))
            .transpose()?,
        Some(FieldValue::Text(encode_string_enum(&after.status)?)),
    ));
    revision = revision.with_field(FieldDelta::new(
        "tool_name",
        before.map(|call| FieldValue::Text(call.request.tool_name.as_str().to_string())),
        Some(FieldValue::Text(
            after.request.tool_name.as_str().to_string(),
        )),
    ));
    Ok(revision)
}

fn agent_tool_result_revision(
    before: Option<&AgentToolResult>,
    after: &AgentToolResult,
    event_id: eidetic_core::contracts::ChangeEventId,
    operation: &RevisionOperation,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::AgentToolResult,
        after.call_id.0.to_string(),
        event_id,
        operation.clone(),
    )
    .with_field(FieldDelta::new(
        "status",
        before
            .map(|result| encode_string_enum(&result.status).map(FieldValue::Text))
            .transpose()?,
        Some(FieldValue::Text(encode_string_enum(&after.status)?)),
    )))
}

fn row_to_agent_run(row: &Row<'_>) -> Result<AgentRun, rusqlite::Error> {
    let id: String = row.get(0)?;
    let workflow_id: String = row.get(1)?;
    let intent: String = row.get(2)?;
    let status: String = row.get(3)?;
    let created_at_ms: i64 = row.get(4)?;
    let completed_at_ms: Option<i64> = row.get(5)?;

    Ok(AgentRun {
        id: AgentRunId(parse_uuid(row, 0, &id)?),
        workflow_id: eidetic_core::contracts::AgentWorkflowId::new(workflow_id)
            .map_err(|error| conversion_failure(row, 1, error))?,
        intent: decode_string_enum(row, 2, &intent)?,
        status: decode_string_enum(row, 3, &status)?,
        created_at_ms: u64_from_i64(row, 4, created_at_ms)?,
        completed_at_ms: completed_at_ms
            .map(|value| u64_from_i64(row, 5, value))
            .transpose()?,
        error: row.get(6)?,
    })
}

fn row_to_agent_tool_call(row: &Row<'_>) -> Result<AgentToolCall, rusqlite::Error> {
    let id: String = row.get(0)?;
    let run_id: String = row.get(1)?;
    let sequence: i64 = row.get(2)?;
    let tool_name: String = row.get(3)?;
    let arguments_json: String = row.get(4)?;
    let status: String = row.get(5)?;
    let created_at_ms: i64 = row.get(6)?;
    let arguments =
        serde_json::from_str(&arguments_json).map_err(|error| conversion_failure(row, 4, error))?;

    Ok(AgentToolCall {
        id: AgentToolCallId(parse_uuid(row, 0, &id)?),
        run_id: AgentRunId(parse_uuid(row, 1, &run_id)?),
        sequence: u32::try_from(sequence).map_err(|error| conversion_failure(row, 2, error))?,
        request: eidetic_core::contracts::AgentToolRequest {
            tool_name: eidetic_core::contracts::AgentToolName::new(tool_name)
                .map_err(|error| conversion_failure(row, 3, error))?,
            arguments,
        },
        status: decode_string_enum(row, 5, &status)?,
        created_at_ms: u64_from_i64(row, 6, created_at_ms)?,
    })
}

fn row_to_agent_tool_result(row: &Row<'_>) -> Result<AgentToolResult, rusqlite::Error> {
    let call_id: String = row.get(0)?;
    let status: String = row.get(1)?;
    let payload_json: String = row.get(2)?;
    let completed_at_ms: i64 = row.get(3)?;

    Ok(AgentToolResult {
        call_id: AgentToolCallId(parse_uuid(row, 0, &call_id)?),
        status: decode_string_enum(row, 1, &status)?,
        payload: serde_json::from_str(&payload_json)
            .map_err(|error| conversion_failure(row, 2, error))?,
        completed_at_ms: u64_from_i64(row, 3, completed_at_ms)?,
    })
}

fn optional_u64_to_i64(value: Option<u64>) -> Option<i64> {
    value.map(|value| value as i64)
}

fn u64_from_i64(row: &Row<'_>, index: usize, value: i64) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|error| conversion_failure(row, index, error))
}

fn encode_string_enum<T: serde::Serialize>(value: &T) -> Result<String, HistoryStoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected enum to serialize as string".to_string(),
        )),
    }
}

fn decode_string_enum<T: serde::de::DeserializeOwned>(
    row: &Row<'_>,
    index: usize,
    value: &str,
) -> Result<T, rusqlite::Error> {
    serde_json::from_value(serde_json::Value::String(value.to_string()))
        .map_err(|error| conversion_failure(row, index, error))
}

fn parse_uuid(row: &Row<'_>, index: usize, value: &str) -> Result<uuid::Uuid, rusqlite::Error> {
    uuid::Uuid::parse_str(value).map_err(|error| conversion_failure(row, index, error))
}

fn conversion_failure<E>(row: &Row<'_>, index: usize, error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    let value_type = row
        .get_ref(index)
        .map(|value| value.data_type())
        .unwrap_or(rusqlite::types::Type::Null);
    rusqlite::Error::FromSqlConversionFailure(index, value_type, Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        AgentToolArguments, AgentToolName, AgentToolRequest, AgentToolResultPayload,
        AgentWorkflowId, AgentWorkflowIntent, BibleGraphNodeId,
    };

    #[test]
    fn agent_workflow_store_records_run_call_and_result() {
        let mut conn = Connection::open_in_memory().unwrap();
        let run = agent_run(AgentRunStatus::Running);
        let call = agent_tool_call(run.id, AgentToolCallStatus::Completed);
        let result = AgentToolResult {
            call_id: call.id,
            status: AgentToolResultStatus::Succeeded,
            payload: AgentToolResultPayload::Text {
                text: "Harbor context".to_string(),
            },
            completed_at_ms: 30,
        };

        assert_eq!(
            record_agent_run(&mut conn, &CommandEnvelope::new(run.clone())).unwrap(),
            RecordChangeOutcome::Recorded
        );
        assert_eq!(
            record_agent_tool_call(&mut conn, &CommandEnvelope::new(call.clone())).unwrap(),
            RecordChangeOutcome::Recorded
        );
        assert_eq!(
            record_agent_tool_result(&mut conn, &CommandEnvelope::new(result.clone())).unwrap(),
            RecordChangeOutcome::Recorded
        );

        assert_eq!(load_agent_run(&conn, run.id).unwrap(), Some(run.clone()));
        assert_eq!(load_agent_tool_calls(&conn, run.id).unwrap(), vec![call]);
        assert_eq!(
            load_agent_tool_result(&conn, result.call_id).unwrap(),
            Some(result)
        );
    }

    #[test]
    fn duplicate_agent_run_command_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = CommandEnvelope::new(agent_run(AgentRunStatus::Running));

        assert_eq!(
            record_agent_run(&mut conn, &command).unwrap(),
            RecordChangeOutcome::Recorded
        );
        assert_eq!(
            record_agent_run(&mut conn, &command).unwrap(),
            RecordChangeOutcome::AlreadyRecorded
        );
    }

    #[test]
    fn agent_run_status_update_replaces_current_state_and_keeps_history() {
        let mut conn = Connection::open_in_memory().unwrap();
        let mut run = agent_run(AgentRunStatus::Running);
        record_agent_run(&mut conn, &CommandEnvelope::new(run.clone())).unwrap();
        run.status = AgentRunStatus::Completed;
        run.completed_at_ms = Some(100);

        record_agent_run(&mut conn, &CommandEnvelope::new(run.clone())).unwrap();

        assert_eq!(load_agent_run(&conn, run.id).unwrap(), Some(run.clone()));
        let revision_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM object_revisions WHERE object_kind = 'agent_run'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(revision_count, 2);
    }

    #[test]
    fn agent_tool_call_rejects_unknown_run() {
        let mut conn = Connection::open_in_memory().unwrap();
        let call = agent_tool_call(AgentRunId::new(), AgentToolCallStatus::Pending);

        let error = record_agent_tool_call(&mut conn, &CommandEnvelope::new(call)).unwrap_err();

        assert!(matches!(error, HistoryStoreError::InvalidValue(_)));
    }

    fn agent_run(status: AgentRunStatus) -> AgentRun {
        AgentRun {
            id: AgentRunId::new(),
            workflow_id: AgentWorkflowId::new("workflow.premise.graph").unwrap(),
            status,
            intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
            created_at_ms: 10,
            completed_at_ms: None,
            error: None,
        }
    }

    fn agent_tool_call(run_id: AgentRunId, status: AgentToolCallStatus) -> AgentToolCall {
        AgentToolCall {
            id: AgentToolCallId::new(),
            run_id,
            sequence: 1,
            request: AgentToolRequest {
                tool_name: AgentToolName::new("read_bible_node").unwrap(),
                arguments: AgentToolArguments::ReadBibleNode {
                    node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
                },
            },
            status,
            created_at_ms: 20,
        }
    }
}
