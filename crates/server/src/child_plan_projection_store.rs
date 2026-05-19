use eidetic_core::ai::backend::{
    ChildPlan, ChildPlanId, ChildPlanListProjection, ChildPlanRecord, ChildPlanStatus,
    ChildProposal,
};
use eidetic_core::contracts::{ObjectKind, ProjectionEnvelope, ProjectionVersion};
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
use rusqlite::{Connection, Row, params};
use serde::de::DeserializeOwned;

use crate::child_plan_store::{self, ChildPlanStoreError};
use crate::history_store;

pub(crate) fn load_child_plan_list_projection(
    conn: &Connection,
) -> Result<ProjectionEnvelope<ChildPlanListProjection>, ChildPlanStoreError> {
    let plans = load_child_plan_records(conn)?;
    let summary = history_store::load_revision_summary_for_kind(conn, ObjectKind::ChildPlan)?;
    let projection = ChildPlanListProjection { plans };
    Ok(match summary.latest_change_event_id {
        Some(change_event_id) => ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        ),
        None => ProjectionEnvelope::initial(projection),
    })
}

fn load_child_plan_records(conn: &Connection) -> Result<Vec<ChildPlanRecord>, ChildPlanStoreError> {
    child_plan_store::create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT id, parent_node_id, target_child_level, status, created_at_ms
         FROM child_plans
         ORDER BY created_at_ms ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_plan_header)?;
    let mut plans = Vec::new();
    for row in rows {
        let (id, parent_node_id, target_child_level, status, created_at_ms) = row?;
        let children = load_child_plan_children(conn, &id)?;
        plans.push(ChildPlanRecord {
            plan: ChildPlan {
                id,
                parent_node_id,
                target_child_level,
                children,
            },
            status,
            created_at_ms,
        });
    }
    Ok(plans)
}

fn load_child_plan_children(
    conn: &Connection,
    plan_id: &ChildPlanId,
) -> Result<Vec<ChildProposal>, ChildPlanStoreError> {
    let mut statement = conn.prepare(
        "SELECT child_index, name, beat_type, outline, weight, location
         FROM child_plan_children
         WHERE plan_id = ?1
         ORDER BY child_index ASC",
    )?;
    let rows = statement.query_map([plan_id.as_str()], row_to_child)?;
    let mut children = Vec::new();
    for row in rows {
        let (index, mut child) = row?;
        child.characters = load_child_references(conn, plan_id, index, "character")?;
        child.props = load_child_references(conn, plan_id, index, "prop")?;
        children.push(child);
    }
    Ok(children)
}

fn load_child_references(
    conn: &Connection,
    plan_id: &ChildPlanId,
    child_index: u32,
    reference_kind: &str,
) -> Result<Vec<String>, ChildPlanStoreError> {
    let mut statement = conn.prepare(
        "SELECT reference_text
         FROM child_plan_child_references
         WHERE plan_id = ?1 AND child_index = ?2 AND reference_kind = ?3
         ORDER BY sort_order ASC",
    )?;
    let rows = statement.query_map(
        params![plan_id.as_str(), child_index as i64, reference_kind],
        |row| row.get::<_, String>(0),
    )?;
    let mut references = Vec::new();
    for row in rows {
        references.push(row?);
    }
    Ok(references)
}

fn row_to_plan_header(
    row: &Row<'_>,
) -> Result<(ChildPlanId, NodeId, StoryLevel, ChildPlanStatus, u64), rusqlite::Error> {
    let id: String = row.get(0)?;
    let parent_node_id: String = row.get(1)?;
    let target_child_level: String = row.get(2)?;
    let status: String = row.get(3)?;
    let created_at_ms: i64 = row.get(4)?;
    Ok((
        ChildPlanId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        NodeId(uuid::Uuid::parse_str(&parent_node_id).map_err(|e| conversion_failure(row, 1, e))?),
        decode_string_enum(&target_child_level).map_err(|e| conversion_failure(row, 2, e))?,
        decode_string_enum(&status).map_err(|e| conversion_failure(row, 3, e))?,
        u64::try_from(created_at_ms).map_err(|e| conversion_failure(row, 4, e))?,
    ))
}

fn row_to_child(row: &Row<'_>) -> Result<(u32, ChildProposal), rusqlite::Error> {
    let child_index: i64 = row.get(0)?;
    let beat_type: Option<String> = row.get(2)?;
    let weight: f64 = row.get(4)?;
    Ok((
        u32::try_from(child_index).map_err(|e| conversion_failure(row, 0, e))?,
        ChildProposal {
            name: row.get(1)?,
            level: None,
            beat_type: beat_type
                .as_deref()
                .map(decode_string_enum::<BeatType>)
                .transpose()
                .map_err(|e| conversion_failure(row, 2, e))?,
            outline: row.get(3)?,
            weight: weight as f32,
            characters: Vec::new(),
            location: row.get(5)?,
            props: Vec::new(),
        },
    ))
}

fn decode_string_enum<T>(value: &str) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    serde_json::from_value(serde_json::Value::String(value.to_string()))
}

fn conversion_failure<E>(row: &Row<'_>, index: usize, error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    rusqlite::Error::FromSqlConversionFailure(
        index,
        row.get_ref_unwrap(index).data_type(),
        Box::new(error),
    )
}
