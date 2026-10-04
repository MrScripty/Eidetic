use std::collections::BTreeMap;

use eidetic_core::contracts::{
    FieldValue, ObjectKind, ProjectionEnvelope, ProjectionVersion, RevisionOperation,
};
use rusqlite::Connection;
use serde::Serialize;

use crate::history_store::{self, HistoryStoreError};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ObjectFieldProjection {
    pub object_kind: ObjectKind,
    pub object_id: String,
    pub deleted: bool,
    pub fields: BTreeMap<String, FieldValue>,
}

pub(crate) fn load_object_field_projection(
    conn: &Connection,
    object_kind: ObjectKind,
    object_id: &str,
) -> Result<ObjectFieldProjection, HistoryStoreError> {
    let revisions = history_store::load_revisions_for_object(conn, object_kind.clone(), object_id)?;
    Ok(build_projection(object_kind, object_id, &revisions))
}

/// Resolve only the historical evidence bound by the captured event. Later
/// edits/deletion must not alter or invalidate evidence consumed by generation.
pub(crate) fn load_object_field_projection_at_event(
    conn: &Connection,
    object_kind: ObjectKind,
    object_id: &str,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<Option<ObjectFieldProjection>, HistoryStoreError> {
    let revisions = history_store::load_revisions_for_object(conn, object_kind.clone(), object_id)?;
    Ok(revisions
        .iter()
        .rposition(|revision| revision.change_event_id == event_id)
        .map(|index| build_projection(object_kind, object_id, &revisions[..=index])))
}

pub(crate) fn load_object_field_projection_envelope(
    conn: &Connection,
    object_kind: ObjectKind,
    object_id: &str,
) -> Result<ProjectionEnvelope<ObjectFieldProjection>, HistoryStoreError> {
    let revisions = history_store::load_revisions_for_object(conn, object_kind.clone(), object_id)?;
    let projection = build_projection(object_kind, object_id, &revisions);

    match revisions.last() {
        Some(revision) => Ok(ProjectionEnvelope::from_event(
            ProjectionVersion(revisions.len() as u64 + 1),
            revision.change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}

fn build_projection(
    object_kind: ObjectKind,
    object_id: &str,
    revisions: &[eidetic_core::contracts::ObjectRevision],
) -> ObjectFieldProjection {
    let mut projection = ObjectFieldProjection {
        object_kind,
        object_id: object_id.to_string(),
        deleted: false,
        fields: BTreeMap::new(),
    };

    for revision in revisions.iter() {
        match &revision.operation {
            RevisionOperation::Create | RevisionOperation::Update => {
                projection.deleted = false;
                for field in &revision.fields {
                    match &field.new_value {
                        Some(value) => {
                            projection
                                .fields
                                .insert(field.field_key.clone(), value.clone());
                        }
                        None => {
                            projection.fields.remove(&field.field_key);
                        }
                    }
                }
            }
            RevisionOperation::Delete => {
                projection.deleted = true;
                projection.fields.clear();
            }
        }
    }

    projection
}

#[cfg(test)]
#[path = "revision_projection_tests.rs"]
mod tests;
