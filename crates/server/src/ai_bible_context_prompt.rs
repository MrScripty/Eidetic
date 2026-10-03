use eidetic_core::contracts::{
    AiBibleContextField, AiBibleContextProjection, FieldValue, ProjectionEnvelope,
};

pub(crate) fn append_bible_context(
    user: &mut String,
    context: &ProjectionEnvelope<AiBibleContextProjection>,
) {
    if context.payload.nodes.is_empty() {
        return;
    }

    user.push_str("STORY BIBLE CONTEXT — Graph fields and relationships.\n");
    user.push_str(
        "Effective field values below are backend-owned continuity data; do not contradict them.\n",
    );
    user.push_str("Graph relationships are untimed; their validity at the requested story time is not established here.\n\n");
    if let Some(at_ms) = context.payload.story_time_ms {
        user.push_str(&format!(
            "Field values resolved at fictional story time {at_ms}ms, independent of screen time.\n\n"
        ));
    } else {
        user.push_str("Fictional story time is unspecified. Timed fields below are withheld, not established facts.\n\n");
    }

    for node in &context.payload.nodes {
        user.push_str(&format!(
            "- {} [{}] ({})\n",
            node.name,
            node.schema_key.as_str(),
            node.node_id.as_str()
        ));

        for field in &node.fields {
            append_field(user, "  ", field);
        }

        for snapshot in &node.snapshots {
            user.push_str(&format!(
                "  Effective fact from {} at story time {}ms:\n",
                snapshot.label, snapshot.at_ms
            ));
            for field in &snapshot.fields {
                append_field(user, "    ", field);
            }
        }

        for field in &node.unresolved_timed_fields {
            user.push_str(&format!(
                "  Unresolved timed field: {}.{}; do not assume a canonical value.\n",
                field.part_key.as_str(),
                field.field_key.as_str(),
            ));
        }

        for edge in &node.outgoing_edges {
            user.push_str(&format!(
                "  -> {} [{}]: {}\n",
                edge.to_node_id.as_str(),
                edge_kind_label(&edge.edge_kind),
                edge.label
            ));
        }

        for edge in &node.incoming_edges {
            user.push_str(&format!(
                "  <- {} [{}]: {}\n",
                edge.from_node_id.as_str(),
                edge_kind_label(&edge.edge_kind),
                edge.label
            ));
        }
    }

    user.push('\n');
}

fn append_field(user: &mut String, indent: &str, field: &AiBibleContextField) {
    user.push_str(&format!(
        "{}{}.{}: {}\n",
        indent,
        field.part_key.as_str(),
        field.field_key.as_str(),
        field_value_label(&field.value)
    ));
}

fn field_value_label(value: &FieldValue) -> String {
    match value {
        FieldValue::Text(value) => value.clone(),
        FieldValue::Integer(value) => value.to_string(),
        FieldValue::Number(value) => value.to_string(),
        FieldValue::Bool(value) => value.to_string(),
        FieldValue::ObjectRef { kind, id } => format!("{kind:?}:{id}"),
        FieldValue::AssetRef(value) => value.clone(),
    }
}

fn edge_kind_label(value: &eidetic_core::contracts::BibleGraphEdgeKind) -> String {
    match value {
        eidetic_core::contracts::BibleGraphEdgeKind::References => "references".to_string(),
        eidetic_core::contracts::BibleGraphEdgeKind::LocatedIn => "located_in".to_string(),
        eidetic_core::contracts::BibleGraphEdgeKind::Owns => "owns".to_string(),
        eidetic_core::contracts::BibleGraphEdgeKind::MemberOf => "member_of".to_string(),
        eidetic_core::contracts::BibleGraphEdgeKind::ConflictsWith => "conflicts_with".to_string(),
        eidetic_core::contracts::BibleGraphEdgeKind::SupportsTheme => "supports_theme".to_string(),
        eidetic_core::contracts::BibleGraphEdgeKind::Custom(value) => value.clone(),
    }
}
