use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, BibleGraphNode, BibleGraphNodeId, BibleReferenceKind,
    BibleReferenceProposal, CanonicalBibleRoot, ChangeEvent, ChangeEventKind, CommandEnvelope,
    FieldDelta, FieldValue, ObjectKind, ObjectRevision, RevisionOperation, SemanticProposalStatus,
};
use rusqlite::Connection;
use serde::Serialize;

use crate::bible_graph_store;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::semantic_proposal_store::{self, SemanticProposalStoreError};

pub(crate) fn record_accept_bible_reference_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<AcceptBibleReferenceProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, SemanticProposalStoreError> {
    semantic_proposal_store::create_schema(conn)?;
    bible_graph_store::create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.bible_reference_accept")?
    {
        return Ok(outcome);
    }
    let proposal =
        semantic_proposal_store::load_bible_reference_proposal(conn, &command.payload.proposal_id)?
            .ok_or_else(|| {
                SemanticProposalStoreError::NotFound(format!(
                    "semantic proposal not found: {}",
                    command.payload.proposal_id.as_str()
                ))
            })?;
    if proposal.status != SemanticProposalStatus::Pending {
        return Err(SemanticProposalStoreError::InvalidCommand(format!(
            "semantic proposal is not pending: {}",
            proposal.id.as_str()
        )));
    }

    let target = accept_target(conn, &proposal, &command.payload)?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalAccepted,
        format!("accept bible reference {}", proposal.reference_text),
    )
    .with_created_at_ms(created_at_ms);
    let proposal_revision = proposal_accept_revision(&proposal, target.node_id(), event.id)?;
    let revisions = match &target {
        AcceptTarget::Create(node) => vec![
            proposal_revision,
            accepted_bible_node_revision(node, event.id),
        ],
        AcceptTarget::LinkExisting(_) => vec![proposal_revision],
    };

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.bible_reference_accept",
        &event,
        &revisions,
        |tx| {
            semantic_proposal_store::update_proposal_status_in_transaction(
                tx,
                &proposal.id,
                SemanticProposalStatus::Pending,
                SemanticProposalStatus::Accepted,
            )?;
            match &target {
                AcceptTarget::Create(node) => {
                    bible_graph_store::insert_node_in_transaction(tx, node, event.id)
                }
                AcceptTarget::LinkExisting(_) => Ok(()),
            }
        },
    )?)
}

enum AcceptTarget {
    Create(BibleGraphNode),
    LinkExisting(BibleGraphNodeId),
}

impl AcceptTarget {
    fn node_id(&self) -> &BibleGraphNodeId {
        match self {
            Self::Create(node) => &node.id,
            Self::LinkExisting(node_id) => node_id,
        }
    }
}

fn accept_target(
    conn: &Connection,
    proposal: &BibleReferenceProposal,
    command: &AcceptBibleReferenceProposalCommand,
) -> Result<AcceptTarget, SemanticProposalStoreError> {
    if let Some(existing) = bible_graph_store::load_node_detail_projection(conn, &command.node_id)?
    {
        if existing.node.schema_key != proposal.proposed_schema_key {
            return Err(SemanticProposalStoreError::InvalidCommand(format!(
                "existing bible graph node schema {} does not match proposal schema {}",
                existing.node.schema_key.as_str(),
                proposal.proposed_schema_key.as_str()
            )));
        }
        return Ok(AcceptTarget::LinkExisting(existing.node.id));
    }

    let node = accepted_bible_node(proposal, command)?;
    if let Some(parent_id) = node.parent_id.as_ref() {
        if !bible_graph_store::node_exists(conn, parent_id)? {
            return Err(SemanticProposalStoreError::InvalidCommand(format!(
                "parent bible graph node does not exist: {}",
                parent_id.as_str()
            )));
        }
    }
    Ok(AcceptTarget::Create(node))
}

fn accepted_bible_node(
    proposal: &BibleReferenceProposal,
    command: &AcceptBibleReferenceProposalCommand,
) -> Result<BibleGraphNode, SemanticProposalStoreError> {
    let name = command
        .name
        .as_deref()
        .unwrap_or(proposal.reference_text.as_str())
        .trim();
    if name.is_empty() {
        return Err(SemanticProposalStoreError::InvalidCommand(
            "name is required".to_string(),
        ));
    }
    Ok(BibleGraphNode {
        id: command.node_id.clone(),
        parent_id: Some(
            command
                .parent_id
                .clone()
                .unwrap_or_else(|| default_parent_id(&proposal.reference_kind)),
        ),
        schema_key: proposal.proposed_schema_key.clone(),
        name: name.to_string(),
        system_owned: false,
        sort_order: command.sort_order,
    })
}

fn default_parent_id(reference_kind: &BibleReferenceKind) -> BibleGraphNodeId {
    match reference_kind {
        BibleReferenceKind::Character => CanonicalBibleRoot::Characters.node_id(),
        BibleReferenceKind::Location => CanonicalBibleRoot::Places.node_id(),
        BibleReferenceKind::Prop => CanonicalBibleRoot::Objects.node_id(),
    }
}

fn proposal_accept_revision(
    proposal: &BibleReferenceProposal,
    node_id: &BibleGraphNodeId,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::SemanticProposal,
        proposal.id.as_str().to_string(),
        event_id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "status",
        Some(FieldValue::Text(encode_string_enum(&proposal.status)?)),
        Some(FieldValue::Text(encode_string_enum(
            &SemanticProposalStatus::Accepted,
        )?)),
    ))
    .with_field(FieldDelta::new(
        "acceptance_target_node_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::BibleNode,
            id: node_id.as_str().to_string(),
        }),
    )))
}

fn accepted_bible_node_revision(
    node: &BibleGraphNode,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let mut revision = ObjectRevision::new(
        ObjectKind::BibleNode,
        node.id.as_str(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "name",
        None,
        Some(FieldValue::Text(node.name.clone())),
    ))
    .with_field(FieldDelta::new(
        "schema_key",
        None,
        Some(FieldValue::Text(node.schema_key.as_str().to_string())),
    ))
    .with_field(FieldDelta::new(
        "sort_order",
        None,
        Some(FieldValue::Integer(i64::from(node.sort_order))),
    ));
    if let Some(parent_id) = node.parent_id.as_ref() {
        revision = revision.with_field(FieldDelta::new(
            "parent_id",
            None,
            Some(FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: parent_id.as_str().to_string(),
            }),
        ));
    }
    revision
}

fn encode_string_enum<T>(value: &T) -> Result<String, HistoryStoreError>
where
    T: Serialize,
{
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected enum to serialize as string".to_string(),
        )),
    }
}
