//! Author-selected baseline supplementation within the existing proposal read.
use std::collections::BTreeSet;

use eidetic_core::contracts::*;
use rusqlite::Connection;

use crate::history_store::HistoryStoreError;

fn refused() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Selected recall evidence is stale or unavailable. Recall again and select up to eight resolved baseline facts at unspecified story time; timed, unresolved and omitted facts cannot be used.".into(),
    )
}

/// The caller owns the same snapshot as all other proposal evidence. A selection
/// adds only chosen values and the names/untimed paths actually supplied with them.
pub(crate) fn append(
    conn: &Connection,
    request: &RequestScriptImpactProposalCommand,
    context: &mut AiBibleContextProjection,
) -> Result<(), HistoryStoreError> {
    let Some(selection) = &request.recall_selection else {
        return Ok(());
    };
    if selection.facts.is_empty() {
        // Explicit empty intent has exactly the existing no-selection behavior.
        // Associated selectors are not consumed evidence or prompt content.
        return Ok(());
    }
    if selection.facts.len() > 8
        || selection.names.len() > 9
        || selection.paths.len() > 32
        || selection.query.story_time_ms.is_some()
        || request.story_time_ms.is_some()
    {
        return Err(refused());
    }
    let recall = crate::bible_recall_projection::load(conn, &selection.query)?.payload;
    let mut chosen = BTreeSet::new();
    let mut nodes = BTreeSet::new();
    let mut fields = Vec::new();
    for fact in &selection.facts {
        if !chosen.insert(fact.field_id.clone()) {
            return Err(refused());
        }
        let node = recall
            .nodes
            .iter()
            .find(|node| node.node_id == fact.node_id)
            .ok_or_else(refused)?;
        let field = node
            .fields
            .iter()
            .find(|field| field.part_key == fact.part_key && field.field_key == fact.field_key)
            .ok_or_else(refused)?;
        if field.source
            != (BibleRecallFieldSource::Baseline {
                field_id: fact.field_id.clone(),
                revision_event_id: fact.revision_event_id,
            })
        {
            return Err(refused());
        }
        let detail = crate::bible_graph_store::load_node_detail_projection(conn, &fact.node_id)?
            .ok_or_else(refused)?;
        let part = detail
            .parts
            .iter()
            .find(|part| part.part.part_key == fact.part_key)
            .ok_or_else(refused)?;
        fields.push((
            fact.node_id.clone(),
            AiBibleContextField {
                part_key: fact.part_key.clone(),
                part_name: part.part.name.clone(),
                field_key: fact.field_key.clone(),
                value: field.value.clone(),
            },
        ));
        nodes.insert(fact.node_id.clone());
    }
    // Include all returned connecting paths for chosen peers, never paths to
    // unselected peers. Their endpoint names are part of the prompt evidence.
    let paths: Vec<_> = recall
        .paths
        .iter()
        .filter(|path| nodes.contains(&path.neighbor_node_id))
        .cloned()
        .collect();
    if !paths.is_empty() {
        nodes.insert(recall.request.anchor_node_id.clone());
    }
    let mut names = Vec::new();
    for id in &nodes {
        let node = recall
            .nodes
            .iter()
            .find(|node| &node.node_id == id)
            .ok_or_else(refused)?;
        names.push(BibleNodeNameInput {
            node_id: id.clone(),
            name: node.name.clone(),
            revision_event_id: node.name_revision_event_id.ok_or_else(refused)?,
        });
    }
    let mut expected_names = selection.names.clone();
    expected_names.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    let mut expected_paths = selection.paths.clone();
    expected_paths.sort_by(|a, b| {
        a.relationship
            .edge
            .edge_id
            .cmp(&b.relationship.edge.edge_id)
    });
    let mut actual_paths = paths.clone();
    actual_paths.sort_by(|a, b| {
        a.relationship
            .edge
            .edge_id
            .cmp(&b.relationship.edge.edge_id)
    });
    if expected_names != names || expected_paths != actual_paths {
        return Err(refused());
    }
    for id in nodes {
        if !context.nodes.iter().any(|node| node.node_id == id) {
            let node = recall
                .nodes
                .iter()
                .find(|node| node.node_id == id)
                .ok_or_else(refused)?;
            context.nodes.push(AiBibleContextNode {
                node_id: id,
                parent_id: None,
                schema_key: node.schema_key.clone(),
                name: node.name.clone(),
                fields: vec![],
                snapshots: vec![],
                unresolved_timed_fields: vec![],
                incoming_edges: vec![],
                outgoing_edges: vec![],
            });
        }
    }
    for (id, field) in fields {
        let node = context
            .nodes
            .iter_mut()
            .find(|node| node.node_id == id)
            .ok_or_else(refused)?;
        if !node.fields.iter().any(|existing| {
            existing.part_key == field.part_key && existing.field_key == field.field_key
        }) {
            node.fields.push(field);
        }
    }
    for path in paths {
        let edge = path.relationship.edge;
        for node in &mut context.nodes {
            for (matches, edges) in [
                (node.node_id == edge.from_node_id, &mut node.outgoing_edges),
                (node.node_id == edge.to_node_id, &mut node.incoming_edges),
            ] {
                if matches
                    && !edges
                        .iter()
                        .any(|existing| existing.edge_id == edge.edge_id)
                {
                    edges.push(edge.clone());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "script_recall_selection_tests.rs"]
mod tests;
