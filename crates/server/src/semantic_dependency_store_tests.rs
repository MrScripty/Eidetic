use eidetic_core::contracts::{
    BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, CommandEnvelope, ObjectKind,
    RecordSemanticDependencyCommand, ScriptSegmentId, SemanticDependency,
    SemanticDependencyEndpoint, SemanticDependencyId, SemanticDependencyKind,
};

use super::{
    DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
    SemanticDependencyStoreError, load_semantic_dependency_projection, record_semantic_dependency,
};
use crate::history_store::{self, RecordChangeOutcome};

#[test]
fn records_and_projects_dependencies_by_target_field() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let command = dependency_command("dependency.weather.scene");

    let outcome = record_semantic_dependency(&mut conn, &command, 100).unwrap();
    let projection = load_semantic_dependency_projection(
        &conn,
        &SemanticDependencyFilter {
            endpoint: DependencyEndpointFilter {
                kind: "bible_field".to_string(),
                id: "node.location.harbor".to_string(),
                part_key: Some("weather".to_string()),
                field_key: Some("current".to_string()),
            },
            direction: DependencyDirection::Target,
        },
    )
    .unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(projection.version.0, 2);
    assert_eq!(projection.payload.dependencies.len(), 1);
    assert_eq!(
        projection.payload.dependencies[0].id.as_str(),
        "dependency.weather.scene"
    );
    assert_eq!(
        projection.payload.dependencies[0].target,
        bible_field_endpoint()
    );
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticDependency,
        "dependency.weather.scene",
    )
    .unwrap();
    assert_eq!(revisions.len(), 1);
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "confidence")
    );
}

#[test]
fn replays_duplicate_dependency_command() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let command = dependency_command("dependency.weather.replay");

    let first = record_semantic_dependency(&mut conn, &command, 100).unwrap();
    let second = record_semantic_dependency(&mut conn, &command, 100).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
}

#[test]
fn rejects_reused_dependency_id_from_distinct_command() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let first = dependency_command("dependency.weather.duplicate");
    let mut second = dependency_command("dependency.weather.duplicate");
    second.id = eidetic_core::contracts::CommandId::new();

    record_semantic_dependency(&mut conn, &first, 100).unwrap();
    let error = record_semantic_dependency(&mut conn, &second, 101).unwrap_err();

    assert!(
        matches!(error, SemanticDependencyStoreError::InvalidCommand(message) if message.contains("already exists"))
    );
}

#[test]
fn rejects_out_of_range_confidence() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let mut command = dependency_command("dependency.weather.invalid-confidence");
    command.payload.dependency.confidence = Some(1.5);

    let error = record_semantic_dependency(&mut conn, &command, 100).unwrap_err();

    assert!(
        matches!(error, SemanticDependencyStoreError::InvalidCommand(message) if message.contains("confidence"))
    );
}

fn dependency_command(id: &str) -> CommandEnvelope<RecordSemanticDependencyCommand> {
    CommandEnvelope::new(RecordSemanticDependencyCommand {
        dependency: SemanticDependency {
            id: SemanticDependencyId::new(id).unwrap(),
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: ScriptSegmentId::new("script.segment.scene-1").unwrap(),
            },
            target: bible_field_endpoint(),
            kind: SemanticDependencyKind::UsesFact,
            rationale: Some("Scene states the harbor weather.".to_string()),
            confidence: Some(0.88),
            created_at_ms: 100,
        },
    })
}

fn bible_field_endpoint() -> SemanticDependencyEndpoint {
    SemanticDependencyEndpoint::BibleField {
        node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
        part_key: BibleGraphPartKey::new("weather").unwrap(),
        field_key: BibleGraphFieldKey::new("current").unwrap(),
        field_id: None,
    }
}
