use std::collections::{BTreeMap, BTreeSet};

use eidetic_core::contracts::{
    AiBibleContextField, AiBibleContextFieldRef, AiBibleContextSnapshot, BibleGraphSnapshotField,
    BibleGraphSnapshotProjection,
};

use crate::history_store::HistoryStoreError;

pub(crate) struct ResolvedFields {
    pub fields: Vec<AiBibleContextField>,
    pub snapshots: Vec<AiBibleContextSnapshot>,
    pub unresolved_fields: Vec<AiBibleContextFieldRef>,
    /// Exact selected assertion; current callers continue using the same values.
    pub snapshot_sources: BTreeMap<(String, String), ResolvedSnapshotSource>,
}

pub(crate) struct ResolvedSnapshotSource {
    pub field: BibleGraphSnapshotField,
    pub at_ms: u64,
    pub label: String,
}

type FieldKey = (String, String);

struct LatestAssertions {
    at_ms: u64,
    candidates: Vec<(String, BibleGraphSnapshotField)>,
}

/// Resolve sparse fictional-time assertions independently of clip placement.
/// A null value explicitly withholds the field until a later assertion; a
/// missing field inherits its previous value. Equal-time disagreements fail
/// closed rather than using edit order, graph sort order or model judgement.
pub(crate) fn resolve_fields(
    mut fields: Vec<AiBibleContextField>,
    snapshots: Vec<BibleGraphSnapshotProjection>,
    story_time_ms: Option<u64>,
) -> Result<ResolvedFields, HistoryStoreError> {
    let baseline_keys: BTreeSet<_> = fields
        .iter()
        .map(|field| {
            (
                field.part_key.as_str().to_owned(),
                field.field_key.as_str().to_owned(),
            )
        })
        .collect();
    let mut timed_keys = BTreeSet::new();
    let mut latest: BTreeMap<FieldKey, LatestAssertions> = BTreeMap::new();
    let mut unresolved: BTreeMap<(String, String), AiBibleContextFieldRef> = BTreeMap::new();
    for projection in snapshots {
        for field in projection.fields {
            let key = (
                field.part_key.as_str().to_owned(),
                field.field_key.as_str().to_owned(),
            );
            if story_time_ms.is_none() {
                timed_keys.insert(key.clone());
                unresolved.insert(
                    key,
                    AiBibleContextFieldRef {
                        part_key: field.part_key,
                        field_key: field.field_key,
                    },
                );
                continue;
            }
            let at_ms = projection.snapshot.at_ms;
            if at_ms > story_time_ms.expect("time checked above") {
                // A future assertion establishes that the field exists, not
                // that its value is already true. Keep unknown identity when
                // neither a baseline nor an eligible assertion supplies it.
                if !baseline_keys.contains(&key) {
                    unresolved.insert(
                        key,
                        AiBibleContextFieldRef {
                            part_key: field.part_key,
                            field_key: field.field_key,
                        },
                    );
                }
                continue;
            }
            timed_keys.insert(key.clone());
            match latest.get_mut(&key) {
                Some(previous) if previous.at_ms == at_ms => {
                    previous
                        .candidates
                        .push((projection.snapshot.label.clone(), field));
                }
                Some(previous) if previous.at_ms > at_ms => {}
                _ => {
                    latest.insert(
                        key,
                        LatestAssertions {
                            at_ms,
                            candidates: vec![(projection.snapshot.label.clone(), field)],
                        },
                    );
                }
            }
        }
    }
    fields.retain(|field| {
        !timed_keys.contains(&(
            field.part_key.as_str().to_owned(),
            field.field_key.as_str().to_owned(),
        ))
    });
    let mut effective: BTreeMap<(u64, String), Vec<AiBibleContextField>> = BTreeMap::new();
    let mut snapshot_sources = BTreeMap::new();
    for (
        key,
        LatestAssertions {
            at_ms,
            mut candidates,
        },
    ) in latest
    {
        // Eligible facts take precedence over any future-only unknown marker.
        unresolved.remove(&key);
        candidates.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.id.as_str().cmp(b.1.id.as_str()))
        });
        let (label, field) = candidates.remove(0);
        if candidates
            .iter()
            .any(|(_, other)| other.value != field.value)
        {
            return Err(HistoryStoreError::InvalidValue(format!(
                "conflicting story facts for {}.{} at story time {at_ms}ms",
                key.0, key.1,
            )));
        }
        if let Some(value) = field.value.clone() {
            snapshot_sources.insert(
                key,
                ResolvedSnapshotSource {
                    field: field.clone(),
                    at_ms,
                    label: label.clone(),
                },
            );
            effective
                .entry((at_ms, label))
                .or_default()
                .push(AiBibleContextField {
                    part_key: field.part_key,
                    part_name: field.part_name,
                    field_key: field.field_key,
                    value,
                });
        } else {
            unresolved.insert(
                key,
                AiBibleContextFieldRef {
                    part_key: field.part_key,
                    field_key: field.field_key,
                },
            );
        }
    }
    Ok(ResolvedFields {
        fields,
        snapshots: effective
            .into_iter()
            .map(|((at_ms, label), fields)| AiBibleContextSnapshot {
                label,
                at_ms,
                fields,
            })
            .collect(),
        unresolved_fields: unresolved.into_values().collect(),
        snapshot_sources,
    })
}

#[cfg(test)]
#[path = "ai_temporal_context_tests.rs"]
mod tests;
