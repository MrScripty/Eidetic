use super::*;
use eidetic_core::contracts::{
    BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, BibleGraphSnapshot,
    BibleGraphSnapshotFieldId, BibleGraphSnapshotId, FieldValue,
};

fn snapshot(
    label: &str,
    at_ms: u64,
    key: &str,
    value: Option<&str>,
) -> BibleGraphSnapshotProjection {
    let id = BibleGraphSnapshotId::new(format!("snapshot.{label}")).unwrap();
    BibleGraphSnapshotProjection {
        snapshot: BibleGraphSnapshot {
            id: id.clone(),
            node_id: BibleGraphNodeId::new("node.ada").unwrap(),
            at_ms,
            label: label.into(),
            sort_order: 0,
        },
        fields: vec![BibleGraphSnapshotField {
            id: BibleGraphSnapshotFieldId::new(format!("field.{label}.{key}")).unwrap(),
            snapshot_id: id,
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            field_key: BibleGraphFieldKey::new(key).unwrap(),
            value: value.map(|value| FieldValue::Text(value.into())),
            sort_order: 0,
        }],
    }
}

#[test]
fn sparse_snapshots_resolve_latest_per_field_independent_of_input_order() {
    let facts = vec![
        snapshot("rain", 100, "weather", Some("rain")),
        snapshot("door", 200, "door", Some("open")),
        snapshot("sun", 300, "weather", Some("sun")),
    ];
    let resolved = resolve_fields(vec![], facts.clone(), Some(250)).unwrap();
    let reversed = resolve_fields(vec![], facts.into_iter().rev().collect(), Some(250)).unwrap();
    assert_eq!(resolved.snapshots, reversed.snapshots);
    assert_eq!(resolved.snapshots.len(), 2);
    assert_eq!(
        resolved.snapshots[0].fields[0].value,
        FieldValue::Text("rain".into())
    );
    assert_eq!(
        resolved.snapshots[1].fields[0].value,
        FieldValue::Text("open".into())
    );
}

#[test]
fn same_time_conflict_is_rejected_and_later_resolution_supersedes_it() {
    let mut facts = vec![
        snapshot("rain", 100, "weather", Some("rain")),
        snapshot("sun", 100, "weather", Some("sun")),
    ];
    assert!(resolve_fields(vec![], facts.clone(), Some(100)).is_err());
    facts.push(snapshot("snow", 200, "weather", Some("snow")));
    let resolved = resolve_fields(vec![], facts.clone(), Some(200)).unwrap();
    let reversed = resolve_fields(vec![], facts.into_iter().rev().collect(), Some(200)).unwrap();
    assert_eq!(resolved.snapshots, reversed.snapshots);
    assert_eq!(
        resolved.snapshots[0].fields[0].value,
        FieldValue::Text("snow".into())
    );
}

#[test]
fn duplicate_same_time_facts_coalesce_with_stable_provenance() {
    let facts = vec![
        snapshot("z", 100, "weather", Some("rain")),
        snapshot("a", 100, "weather", Some("rain")),
    ];
    let resolved = resolve_fields(vec![], facts, Some(100)).unwrap();
    assert_eq!(resolved.snapshots.len(), 1);
    assert_eq!(resolved.snapshots[0].label, "a");
}

#[test]
fn cleared_fact_is_unknown_without_resurrecting_earlier_assertion() {
    let facts = vec![
        snapshot("rain", 100, "weather", Some("rain")),
        snapshot("clear", 200, "weather", None),
    ];
    let resolved = resolve_fields(vec![], facts, Some(200)).unwrap();
    assert!(resolved.snapshots.is_empty());
    assert_eq!(resolved.unresolved_fields[0].field_key.as_str(), "weather");
}

#[test]
fn future_only_field_stays_unknown_alongside_an_effective_field() {
    let facts = vec![
        snapshot("door", 50, "door", Some("open")),
        snapshot("rain", 100, "weather", Some("rain")),
    ];
    let before = resolve_fields(vec![], facts.clone(), Some(99)).unwrap();
    let reversed =
        resolve_fields(vec![], facts.clone().into_iter().rev().collect(), Some(99)).unwrap();
    assert_eq!(before.snapshots, reversed.snapshots);
    assert_eq!(before.unresolved_fields, reversed.unresolved_fields);
    assert_eq!(before.snapshots.len(), 1);
    assert_eq!(
        before.snapshots[0].fields[0].value,
        FieldValue::Text("open".into())
    );
    assert_eq!(before.unresolved_fields.len(), 1);
    assert_eq!(before.unresolved_fields[0].field_key.as_str(), "weather");
    assert!(before.fields.is_empty());

    let at = resolve_fields(vec![], facts, Some(100)).unwrap();
    assert!(at.unresolved_fields.is_empty());
    assert_eq!(at.snapshots.len(), 2);
}

#[test]
fn future_assertion_does_not_hide_an_existing_baseline() {
    let baseline = AiBibleContextField {
        part_key: BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".into(),
        field_key: BibleGraphFieldKey::new("weather").unwrap(),
        value: FieldValue::Text("dry".into()),
    };
    let resolved = resolve_fields(
        vec![baseline.clone()],
        vec![snapshot("rain", 100, "weather", Some("rain"))],
        Some(99),
    )
    .unwrap();
    assert_eq!(resolved.fields, vec![baseline]);
    assert!(resolved.snapshots.is_empty());
    assert!(resolved.unresolved_fields.is_empty());
}
