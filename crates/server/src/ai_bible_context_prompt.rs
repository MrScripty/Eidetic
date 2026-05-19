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

    user.push_str("STORY BIBLE CONTEXT — Established graph facts.\n");
    user.push_str("These facts are backend-owned continuity data; do not contradict them:\n\n");

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
                "  Snapshot: {} @ {}ms\n",
                snapshot.label, snapshot.at_ms
            ));
            for field in &snapshot.fields {
                append_field(user, "    ", field);
            }
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
