use eidetic_core::contracts::{ChangeReviewProjection, ProjectionEnvelope};
use rusqlite::{Connection, OptionalExtension};

use crate::history_store::{self, HistoryStoreError};

const DEFAULT_CHANGE_REVIEW_LIMIT: u32 = 100;

pub(crate) fn load_change_review_projection_envelope(
    conn: &Connection,
) -> Result<ProjectionEnvelope<ChangeReviewProjection>, HistoryStoreError> {
    history_store::create_schema(conn)?;
    let changes = history_store::load_change_review_changes(conn, DEFAULT_CHANGE_REVIEW_LIMIT)?;
    let projection = ChangeReviewProjection { changes };
    let summary = load_revision_summary(conn)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(ProjectionEnvelope::from_event(
            eidetic_core::contracts::ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}

fn load_revision_summary(
    conn: &Connection,
) -> Result<history_store::RevisionSummary, HistoryStoreError> {
    let revision_count = conn.query_row("SELECT COUNT(*) FROM object_revisions", [], |row| {
        row.get::<_, i64>(0)
    })?;
    let latest_change_event_id = conn
        .query_row(
            "SELECT change_event_id
             FROM object_revisions
             ORDER BY rowid DESC
             LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|id| {
            uuid::Uuid::parse_str(&id)
                .map(eidetic_core::contracts::ChangeEventId)
                .map_err(|e| HistoryStoreError::InvalidId(e.to_string()))
        })
        .transpose()?;

    Ok(history_store::RevisionSummary {
        revision_count: u64::try_from(revision_count).unwrap_or_default(),
        latest_change_event_id,
    })
}

#[cfg(test)]
#[path = "change_review_projection_tests.rs"]
mod tests;
