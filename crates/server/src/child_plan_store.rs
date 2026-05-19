use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildPlanStatus, ChildProposal};
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectKind,
    ObjectRevision, RevisionOperation,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

const CHILD_PLAN_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS child_plans (
    id                 TEXT PRIMARY KEY CHECK (id <> ''),
    parent_node_id     TEXT NOT NULL CHECK (parent_node_id <> ''),
    target_child_level TEXT NOT NULL CHECK (target_child_level <> ''),
    status             TEXT NOT NULL CHECK (status <> ''),
    created_at_ms      INTEGER NOT NULL,
    created_event_id   TEXT NOT NULL REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_child_plans_parent
    ON child_plans(parent_node_id, created_at_ms);

CREATE TABLE IF NOT EXISTS child_plan_children (
    plan_id     TEXT NOT NULL REFERENCES child_plans(id) ON DELETE CASCADE,
    child_index INTEGER NOT NULL,
    name        TEXT NOT NULL CHECK (name <> ''),
    beat_type   TEXT,
    outline     TEXT NOT NULL CHECK (outline <> ''),
    weight      REAL NOT NULL,
    location    TEXT,
    PRIMARY KEY (plan_id, child_index)
);

CREATE TABLE IF NOT EXISTS child_plan_child_references (
    plan_id        TEXT NOT NULL,
    child_index    INTEGER NOT NULL,
    reference_kind TEXT NOT NULL CHECK (reference_kind <> ''),
    reference_text TEXT NOT NULL CHECK (reference_text <> ''),
    sort_order     INTEGER NOT NULL,
    PRIMARY KEY (plan_id, child_index, reference_kind, sort_order),
    FOREIGN KEY (plan_id, child_index)
        REFERENCES child_plan_children(plan_id, child_index)
        ON DELETE CASCADE
);
"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CreateChildPlanCommand {
    plan_id: ChildPlanId,
    parent_node_id: NodeId,
    target_child_level: StoryLevel,
}

pub(crate) fn create_schema(conn: &Connection) -> Result<(), ChildPlanStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(CHILD_PLAN_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn record_child_plan(
    conn: &mut Connection,
    plan: &ChildPlan,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, ChildPlanStoreError> {
    create_schema(conn)?;
    validate_child_plan(plan)?;
    if plan_exists(conn, &plan.id)? {
        return Err(ChildPlanStoreError::InvalidCommand(format!(
            "child plan already exists: {}",
            plan.id.as_str()
        )));
    }

    let command = CommandEnvelope::new(CreateChildPlanCommand {
        plan_id: plan.id.clone(),
        parent_node_id: plan.parent_node_id,
        target_child_level: plan.target_child_level,
    });
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        format!("create child plan for {}", plan.parent_node_id.0),
    )
    .with_created_at_ms(created_at_ms);
    let revision = child_plan_revision(plan, event.id)?;

    Ok(history_store::record_change_with(
        conn,
        &command,
        "ai.child_plan_create",
        &event,
        &[revision],
        |tx| insert_child_plan_in_transaction(tx, plan, created_at_ms, event.id),
    )?)
}

pub(crate) fn validate_child_plan_for_apply(
    conn: &Connection,
    plan_id: &ChildPlanId,
    parent_node_id: NodeId,
    target_child_level: StoryLevel,
) -> Result<(), ChildPlanStoreError> {
    create_schema(conn)?;
    let Some((stored_parent_id, stored_target_level, stored_status)) = conn
        .query_row(
            "SELECT parent_node_id, target_child_level, status FROM child_plans WHERE id = ?1",
            [plan_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?
    else {
        return Err(ChildPlanStoreError::NotFound(format!(
            "child plan not found: {}",
            plan_id.as_str()
        )));
    };
    if stored_parent_id != parent_node_id.0.to_string() {
        return Err(ChildPlanStoreError::InvalidCommand(format!(
            "child plan {} belongs to parent {}, not {}",
            plan_id.as_str(),
            stored_parent_id,
            parent_node_id.0
        )));
    }
    let expected_target_level = encode_string_enum(&target_child_level)?;
    if stored_target_level != expected_target_level {
        return Err(ChildPlanStoreError::InvalidCommand(format!(
            "child plan {} targets {}, not {}",
            plan_id.as_str(),
            stored_target_level,
            expected_target_level
        )));
    }
    let expected_status = encode_string_enum(&ChildPlanStatus::Pending)?;
    if stored_status != expected_status {
        return Err(ChildPlanStoreError::InvalidCommand(format!(
            "child plan {} is not pending",
            plan_id.as_str()
        )));
    }
    Ok(())
}

pub(crate) fn validate_child_plan_pending(
    conn: &Connection,
    plan_id: &ChildPlanId,
) -> Result<(), ChildPlanStoreError> {
    create_schema(conn)?;
    let Some(stored_status) = conn
        .query_row(
            "SELECT status FROM child_plans WHERE id = ?1",
            [plan_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    else {
        return Err(ChildPlanStoreError::NotFound(format!(
            "child plan not found: {}",
            plan_id.as_str()
        )));
    };
    let expected_status = encode_string_enum(&ChildPlanStatus::Pending)?;
    if stored_status != expected_status {
        return Err(ChildPlanStoreError::InvalidCommand(format!(
            "child plan {} is not pending",
            plan_id.as_str()
        )));
    }
    Ok(())
}

pub(crate) fn applied_child_plan_revision(
    plan_id: &ChildPlanId,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::ChildPlan,
        plan_id.as_str().to_string(),
        event_id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "status",
        Some(FieldValue::Text(encode_string_enum(
            &ChildPlanStatus::Pending,
        )?)),
        Some(FieldValue::Text(encode_string_enum(
            &ChildPlanStatus::Applied,
        )?)),
    )))
}

pub(crate) fn rejected_child_plan_revision(
    plan_id: &ChildPlanId,
    reason: Option<&str>,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    let mut revision = ObjectRevision::new(
        ObjectKind::ChildPlan,
        plan_id.as_str().to_string(),
        event_id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "status",
        Some(FieldValue::Text(encode_string_enum(
            &ChildPlanStatus::Pending,
        )?)),
        Some(FieldValue::Text(encode_string_enum(
            &ChildPlanStatus::Rejected,
        )?)),
    ));
    if let Some(reason) = reason {
        revision = revision.with_field(FieldDelta::new(
            "rejection_reason",
            None,
            Some(FieldValue::Text(reason.to_string())),
        ));
    }
    Ok(revision)
}

pub(crate) fn mark_child_plan_applied_in_transaction(
    tx: &Transaction<'_>,
    plan_id: &ChildPlanId,
) -> Result<(), HistoryStoreError> {
    let updated = tx.execute(
        "UPDATE child_plans
         SET status = ?1
         WHERE id = ?2 AND status = ?3",
        params![
            encode_string_enum(&ChildPlanStatus::Applied)?,
            plan_id.as_str(),
            encode_string_enum(&ChildPlanStatus::Pending)?
        ],
    )?;
    if updated != 1 {
        return Err(HistoryStoreError::InvalidValue(format!(
            "child plan status changed before apply: {}",
            plan_id.as_str()
        )));
    }
    Ok(())
}

pub(crate) fn mark_child_plan_rejected_in_transaction(
    tx: &Transaction<'_>,
    plan_id: &ChildPlanId,
) -> Result<(), HistoryStoreError> {
    let updated = tx.execute(
        "UPDATE child_plans
         SET status = ?1
         WHERE id = ?2 AND status = ?3",
        params![
            encode_string_enum(&ChildPlanStatus::Rejected)?,
            plan_id.as_str(),
            encode_string_enum(&ChildPlanStatus::Pending)?
        ],
    )?;
    if updated != 1 {
        return Err(HistoryStoreError::InvalidValue(format!(
            "child plan status changed before reject: {}",
            plan_id.as_str()
        )));
    }
    Ok(())
}

fn validate_child_plan(plan: &ChildPlan) -> Result<(), ChildPlanStoreError> {
    if plan.children.is_empty() {
        return Err(ChildPlanStoreError::InvalidCommand(
            "child plan requires at least one child".to_string(),
        ));
    }
    for child in &plan.children {
        if child.name.trim().is_empty() {
            return Err(ChildPlanStoreError::InvalidCommand(
                "child name is required".to_string(),
            ));
        }
        if child.outline.trim().is_empty() {
            return Err(ChildPlanStoreError::InvalidCommand(
                "child outline is required".to_string(),
            ));
        }
        if !child.weight.is_finite() || child.weight <= 0.0 {
            return Err(ChildPlanStoreError::InvalidCommand(
                "child weight must be a positive finite number".to_string(),
            ));
        }
    }
    Ok(())
}

fn plan_exists(conn: &Connection, plan_id: &ChildPlanId) -> Result<bool, ChildPlanStoreError> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM child_plans WHERE id = ?1",
            [plan_id.as_str()],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

fn child_plan_revision(
    plan: &ChildPlan,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::ChildPlan,
        plan.id.as_str().to_string(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "parent_node_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::TimelineNode,
            id: plan.parent_node_id.0.to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "target_child_level",
        None,
        Some(FieldValue::Text(encode_string_enum(
            &plan.target_child_level,
        )?)),
    ))
    .with_field(FieldDelta::new(
        "child_count",
        None,
        Some(FieldValue::Integer(plan.children.len() as i64)),
    ))
    .with_field(FieldDelta::new(
        "status",
        None,
        Some(FieldValue::Text(encode_string_enum(
            &ChildPlanStatus::Pending,
        )?)),
    )))
}

fn insert_child_plan_in_transaction(
    tx: &Transaction<'_>,
    plan: &ChildPlan,
    created_at_ms: u64,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO child_plans (
            id, parent_node_id, target_child_level, status, created_at_ms, created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            plan.id.as_str(),
            plan.parent_node_id.0.to_string(),
            encode_string_enum(&plan.target_child_level)?,
            encode_string_enum(&ChildPlanStatus::Pending)?,
            created_at_ms as i64,
            event_id.0.to_string()
        ],
    )?;
    for (index, child) in plan.children.iter().enumerate() {
        insert_child_in_transaction(tx, plan, child, index as u32)?;
    }
    Ok(())
}

fn insert_child_in_transaction(
    tx: &Transaction<'_>,
    plan: &ChildPlan,
    child: &ChildProposal,
    child_index: u32,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO child_plan_children (
            plan_id, child_index, name, beat_type, outline, weight, location
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            plan.id.as_str(),
            child_index as i64,
            child.name.trim(),
            optional_string_enum(child.beat_type.as_ref())?,
            child.outline.trim(),
            child.weight as f64,
            child
                .location
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        ],
    )?;
    insert_references_in_transaction(tx, plan, child_index, "character", &child.characters)?;
    insert_references_in_transaction(tx, plan, child_index, "prop", &child.props)?;
    Ok(())
}

fn insert_references_in_transaction(
    tx: &Transaction<'_>,
    plan: &ChildPlan,
    child_index: u32,
    reference_kind: &str,
    references: &[String],
) -> Result<(), HistoryStoreError> {
    for (index, reference) in references
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .enumerate()
    {
        tx.execute(
            "INSERT INTO child_plan_child_references (
                plan_id, child_index, reference_kind, reference_text, sort_order
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                plan.id.as_str(),
                child_index as i64,
                reference_kind,
                reference,
                index as i64
            ],
        )?;
    }
    Ok(())
}

fn optional_string_enum<T>(value: Option<&T>) -> Result<Option<String>, HistoryStoreError>
where
    T: Serialize,
{
    value.map(encode_string_enum).transpose()
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

#[derive(Debug, thiserror::Error)]
pub(crate) enum ChildPlanStoreError {
    #[error("{0}")]
    InvalidCommand(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

#[cfg(test)]
#[path = "child_plan_store_tests.rs"]
mod tests;
