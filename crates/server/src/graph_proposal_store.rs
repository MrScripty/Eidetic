use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, CreateGraphProposalCommand, FieldDelta,
    FieldValue, GraphProposal, GraphProposalAction, GraphProposalId, GraphProposalListProjection,
    GraphProposalTarget, ObjectKind, ObjectRevision, ProjectionEnvelope, ProjectionVersion,
    RevisionOperation,
};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::bible_graph_value_store::SqlGraphFieldValue;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

const GRAPH_PROPOSAL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS graph_proposals (
    id TEXT PRIMARY KEY CHECK (id <> ''),
    action TEXT NOT NULL CHECK (action <> ''),
    target_kind TEXT NOT NULL CHECK (target_kind <> ''),
    target_json TEXT NOT NULL CHECK (target_json <> ''),
    status TEXT NOT NULL CHECK (status <> ''),
    summary TEXT NOT NULL CHECK (summary <> ''),
    proposed_value_type TEXT,
    proposed_value_text TEXT,
    proposed_value_integer INTEGER,
    proposed_value_number REAL,
    proposed_value_bool INTEGER CHECK (proposed_value_bool IS NULL OR proposed_value_bool IN (0, 1)),
    proposed_value_ref_kind TEXT,
    proposed_value_ref_id TEXT,
    proposed_value_asset_ref TEXT,
    rationale TEXT,
    source_agent_run_id TEXT,
    source_tool_call_id TEXT,
    created_at_ms INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_graph_proposals_status
    ON graph_proposals(status, created_at_ms, id);
CREATE INDEX IF NOT EXISTS idx_graph_proposals_target
    ON graph_proposals(target_kind, created_at_ms, id);
CREATE INDEX IF NOT EXISTS idx_graph_proposals_agent_run
    ON graph_proposals(source_agent_run_id, created_at_ms, id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(GRAPH_PROPOSAL_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn record_create_graph_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<CreateGraphProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    validate_create_command(&command.payload)?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "graph.proposal")? {
        return Ok(outcome);
    }
    if graph_proposal_exists(conn, &command.payload.proposal_id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "graph proposal already exists: {}",
            command.payload.proposal_id.as_str()
        )));
    }

    let proposal = command.payload.clone().into_proposal(created_at_ms);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        format!("create graph proposal {}", proposal.summary),
    )
    .with_created_at_ms(created_at_ms);
    let revision = graph_proposal_revision(&proposal, event.id)?;

    history_store::record_change_with(conn, command, "graph.proposal", &event, &[revision], |tx| {
        insert_graph_proposal_in_transaction(tx, &proposal, event.id)
    })
}

pub(crate) fn load_graph_proposals(
    conn: &Connection,
) -> Result<Vec<GraphProposal>, HistoryStoreError> {
    create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT
            id, action, target_json, status, summary,
            proposed_value_type, proposed_value_text, proposed_value_integer,
            proposed_value_number, proposed_value_bool, proposed_value_ref_kind,
            proposed_value_ref_id, proposed_value_asset_ref, rationale,
            source_agent_run_id, source_tool_call_id, created_at_ms
         FROM graph_proposals
         ORDER BY created_at_ms ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_graph_proposal)?;
    let mut proposals = Vec::new();
    for row in rows {
        proposals.push(row?);
    }
    Ok(proposals)
}

#[cfg(test)]
pub(crate) fn load_graph_proposal(
    conn: &Connection,
    proposal_id: &GraphProposalId,
) -> Result<Option<GraphProposal>, HistoryStoreError> {
    create_schema(conn)?;
    conn.query_row(
        "SELECT
            id, action, target_json, status, summary,
            proposed_value_type, proposed_value_text, proposed_value_integer,
            proposed_value_number, proposed_value_bool, proposed_value_ref_kind,
            proposed_value_ref_id, proposed_value_asset_ref, rationale,
            source_agent_run_id, source_tool_call_id, created_at_ms
         FROM graph_proposals
         WHERE id = ?1",
        [proposal_id.as_str()],
        row_to_graph_proposal,
    )
    .optional()
    .map_err(HistoryStoreError::from)
}

pub(crate) fn load_graph_proposal_list_projection(
    conn: &Connection,
) -> Result<ProjectionEnvelope<GraphProposalListProjection>, HistoryStoreError> {
    let proposals = load_graph_proposals(conn)?;
    let summary = history_store::load_revision_summary_for_kind(conn, ObjectKind::GraphProposal)?;
    let projection = GraphProposalListProjection { proposals };
    Ok(match summary.latest_change_event_id {
        Some(change_event_id) => ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        ),
        None => ProjectionEnvelope::initial(projection),
    })
}

fn validate_create_command(command: &CreateGraphProposalCommand) -> Result<(), HistoryStoreError> {
    if command.summary.trim().is_empty() {
        return Err(HistoryStoreError::InvalidValue(
            "graph proposal summary must not be empty".to_string(),
        ));
    }
    validate_action_target(&command.action, &command.target)?;
    if matches!(command.action, GraphProposalAction::SetBibleField)
        && command.proposed_value.is_none()
    {
        return Err(HistoryStoreError::InvalidValue(
            "field graph proposals require a proposed value".to_string(),
        ));
    }
    Ok(())
}

fn validate_action_target(
    action: &GraphProposalAction,
    target: &GraphProposalTarget,
) -> Result<(), HistoryStoreError> {
    let valid = matches!(
        (action, target),
        (
            GraphProposalAction::CreateBibleNode,
            GraphProposalTarget::BibleNode { .. }
        ) | (
            GraphProposalAction::SetBibleField,
            GraphProposalTarget::BibleField { .. }
        ) | (
            GraphProposalAction::CreateBibleEdge,
            GraphProposalTarget::BibleEdge { .. }
        ) | (
            GraphProposalAction::LinkTimelineContext,
            GraphProposalTarget::TimelineContextLink { .. }
        )
    );
    if !valid {
        return Err(HistoryStoreError::InvalidValue(
            "graph proposal action does not match target".to_string(),
        ));
    }
    Ok(())
}

fn graph_proposal_exists(
    conn: &Connection,
    proposal_id: &GraphProposalId,
) -> Result<bool, HistoryStoreError> {
    conn.query_row(
        "SELECT 1 FROM graph_proposals WHERE id = ?1",
        [proposal_id.as_str()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(HistoryStoreError::from)
}

fn insert_graph_proposal_in_transaction(
    tx: &Transaction<'_>,
    proposal: &GraphProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let proposed_value = SqlGraphFieldValue::from_field_value(proposal.proposed_value.as_ref())?;
    tx.execute(
        "INSERT INTO graph_proposals (
            id, action, target_kind, target_json, status, summary,
            proposed_value_type, proposed_value_text, proposed_value_integer,
            proposed_value_number, proposed_value_bool, proposed_value_ref_kind,
            proposed_value_ref_id, proposed_value_asset_ref, rationale,
            source_agent_run_id, source_tool_call_id, created_at_ms, created_event_id
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19
         )",
        params![
            proposal.id.as_str(),
            encode_string_enum(&proposal.action)?,
            target_kind(&proposal.target),
            serde_json::to_string(&proposal.target)?,
            encode_string_enum(&proposal.status)?,
            proposal.summary,
            proposed_value.value_type,
            proposed_value.text,
            proposed_value.integer,
            proposed_value.number,
            proposed_value.bool_value,
            proposed_value.ref_kind,
            proposed_value.ref_id,
            proposed_value.asset_ref,
            proposal.rationale,
            proposal.source_agent_run_id.map(|id| id.0.to_string()),
            proposal.source_tool_call_id.map(|id| id.0.to_string()),
            proposal.created_at_ms as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn graph_proposal_revision(
    proposal: &GraphProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::GraphProposal,
        proposal.id.as_str(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "action",
        None,
        Some(FieldValue::Text(encode_string_enum(&proposal.action)?)),
    ))
    .with_field(FieldDelta::new(
        "status",
        None,
        Some(FieldValue::Text(encode_string_enum(&proposal.status)?)),
    ))
    .with_field(FieldDelta::new(
        "summary",
        None,
        Some(FieldValue::Text(proposal.summary.clone())),
    )))
}

fn row_to_graph_proposal(row: &Row<'_>) -> Result<GraphProposal, rusqlite::Error> {
    let id: String = row.get(0)?;
    let action: String = row.get(1)?;
    let target_json: String = row.get(2)?;
    let status: String = row.get(3)?;
    let created_at_ms: i64 = row.get(16)?;
    let proposed_value = SqlGraphFieldValue {
        value_type: row.get(5)?,
        text: row.get(6)?,
        integer: row.get(7)?,
        number: row.get(8)?,
        bool_value: row.get(9)?,
        ref_kind: row.get(10)?,
        ref_id: row.get(11)?,
        asset_ref: row.get(12)?,
    }
    .into_field_value()
    .map_err(|error| conversion_failure(row, 5, error))?;

    Ok(GraphProposal {
        id: GraphProposalId::new(id).map_err(|error| conversion_failure(row, 0, error))?,
        action: decode_string_enum(row, 1, &action)?,
        target: serde_json::from_str(&target_json)
            .map_err(|error| conversion_failure(row, 2, error))?,
        status: decode_string_enum(row, 3, &status)?,
        summary: row.get(4)?,
        proposed_value,
        rationale: row.get(13)?,
        source_agent_run_id: row
            .get::<_, Option<String>>(14)?
            .map(|value| uuid::Uuid::parse_str(&value).map(eidetic_core::contracts::AgentRunId))
            .transpose()
            .map_err(|error| conversion_failure(row, 14, error))?,
        source_tool_call_id: row
            .get::<_, Option<String>>(15)?
            .map(|value| {
                uuid::Uuid::parse_str(&value).map(eidetic_core::contracts::AgentToolCallId)
            })
            .transpose()
            .map_err(|error| conversion_failure(row, 15, error))?,
        created_at_ms: u64::try_from(created_at_ms)
            .map_err(|error| conversion_failure(row, 16, error))?,
    })
}

fn target_kind(target: &GraphProposalTarget) -> &'static str {
    match target {
        GraphProposalTarget::BibleNode { .. } => "bible_node",
        GraphProposalTarget::BibleField { .. } => "bible_field",
        GraphProposalTarget::BibleEdge { .. } => "bible_edge",
        GraphProposalTarget::TimelineContextLink { .. } => "timeline_context_link",
    }
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
        AgentRunId, AgentToolCallId, BibleGraphEdgeKind, BibleGraphNodeId, BibleGraphPartKey,
        BibleGraphSchemaKey, CommandId, SemanticProposalStatus,
    };
    use eidetic_core::timeline::node::NodeId;

    #[test]
    fn graph_proposal_store_records_reviewable_node_proposal_without_mutating_graph() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: node_proposal_command("proposal.graph.ada"),
        };

        assert_eq!(
            record_create_graph_proposal(&mut conn, &command, 42).unwrap(),
            RecordChangeOutcome::Recorded
        );

        let proposals = load_graph_proposals(&conn).unwrap();
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].id.as_str(), "proposal.graph.ada");
        assert_eq!(proposals[0].status, SemanticProposalStatus::Pending);
        let graph_node_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'bible_graph_nodes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(graph_node_count, 0);
    }

    #[test]
    fn duplicate_graph_proposal_command_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = CommandEnvelope::new(node_proposal_command("proposal.graph.ada"));

        assert_eq!(
            record_create_graph_proposal(&mut conn, &command, 42).unwrap(),
            RecordChangeOutcome::Recorded
        );
        assert_eq!(
            record_create_graph_proposal(&mut conn, &command, 42).unwrap(),
            RecordChangeOutcome::AlreadyRecorded
        );
    }

    #[test]
    fn graph_proposal_store_round_trips_field_value_and_context_link() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = CommandEnvelope::new(CreateGraphProposalCommand {
            proposal_id: GraphProposalId::new("proposal.graph.link").unwrap(),
            action: GraphProposalAction::LinkTimelineContext,
            target: GraphProposalTarget::TimelineContextLink {
                timeline_node_id: NodeId::new(),
                bible_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            },
            summary: "Link Ada to premise".to_string(),
            proposed_value: Some(FieldValue::Text("active context".to_string())),
            rationale: Some("Ada drives the premise.".to_string()),
            source_agent_run_id: Some(AgentRunId::new()),
            source_tool_call_id: Some(AgentToolCallId::new()),
        });

        record_create_graph_proposal(&mut conn, &command, 42).unwrap();

        let proposal = load_graph_proposal(&conn, &command.payload.proposal_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            proposal.proposed_value,
            Some(FieldValue::Text("active context".to_string()))
        );
        assert!(matches!(
            proposal.target,
            GraphProposalTarget::TimelineContextLink { .. }
        ));
    }

    #[test]
    fn graph_proposal_store_rejects_action_target_mismatch() {
        let mut conn = Connection::open_in_memory().unwrap();
        let mut command = node_proposal_command("proposal.graph.bad");
        command.action = GraphProposalAction::CreateBibleEdge;

        let error = record_create_graph_proposal(&mut conn, &CommandEnvelope::new(command), 42)
            .unwrap_err();

        assert!(matches!(error, HistoryStoreError::InvalidValue(_)));
    }

    fn node_proposal_command(id: &str) -> CreateGraphProposalCommand {
        CreateGraphProposalCommand {
            proposal_id: GraphProposalId::new(id).unwrap(),
            action: GraphProposalAction::CreateBibleNode,
            target: GraphProposalTarget::BibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
                parent_id: BibleGraphNodeId::new("canonical.characters").unwrap(),
                schema_key: BibleGraphSchemaKey::new("canonical.character").unwrap(),
                title: "Ada".to_string(),
            },
            summary: "Create Ada".to_string(),
            proposed_value: None,
            rationale: Some("Premise names Ada.".to_string()),
            source_agent_run_id: None,
            source_tool_call_id: None,
        }
    }

    #[allow(dead_code)]
    fn edge_target() -> GraphProposalTarget {
        GraphProposalTarget::BibleEdge {
            edge_id: eidetic_core::contracts::BibleGraphEdgeId::new("edge.ada.premise").unwrap(),
            from_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            to_node_id: BibleGraphNodeId::new("node.theme.identity").unwrap(),
            edge_kind: BibleGraphEdgeKind::References,
            label: "supports".to_string(),
        }
    }

    #[allow(dead_code)]
    fn field_target() -> GraphProposalTarget {
        GraphProposalTarget::BibleField {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            part_key: BibleGraphPartKey::new("identity").unwrap(),
            field_key: eidetic_core::contracts::BibleGraphFieldKey::new("role").unwrap(),
            field_id: None,
        }
    }
}
