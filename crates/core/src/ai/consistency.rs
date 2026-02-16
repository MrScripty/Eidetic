use crate::ai::backend::EditContext;
use crate::error::Result;
use crate::project::Project;
use crate::timeline::node::{NodeId, StoryNode};
use crate::timeline::relationship::RelationshipType;

use super::helpers::gather_surrounding_context;

/// Build an [`EditContext`] for the consistency reaction pipeline.
///
/// With the CRDT model, there is no separate "before" and "after" — the
/// content field holds the current text. The reactive AI pipeline (Phase 4)
/// will replace this with change-based diffing from Y.Doc.
pub fn build_edit_context(project: &Project, node_id: NodeId) -> Result<EditContext> {
    let node = project.timeline.node(node_id)?.clone();

    // In the CRDT model we only have the current content — no "previous" baseline.
    let previous_script = String::new();
    let new_script = node.content.content.clone();

    let surrounding_context = gather_surrounding_context(&project.timeline, node_id);

    Ok(EditContext {
        node,
        previous_script,
        new_script,
        surrounding_context,
    })
}

/// Find node IDs that are downstream of the edited node and might need updates.
///
/// "Downstream" means:
/// - Later sibling nodes at the same level (chronologically after)
/// - All descendants of those later siblings
/// - Nodes connected via Causal relationships from this node
///
/// Locked nodes are excluded (user has taken ownership of their content).
pub fn downstream_node_ids(project: &Project, node_id: NodeId) -> Vec<NodeId> {
    let mut ids = Vec::new();

    let Ok(node) = project.timeline.node(node_id) else {
        return ids;
    };
    let target_end = node.time_range.end_ms;

    // Later siblings at the same level.
    let siblings = project.timeline.siblings_of(node_id);
    for sibling in siblings {
        if sibling.time_range.start_ms >= target_end && !sibling.locked && has_content(sibling) {
            ids.push(sibling.id);
        }
    }

    // Nodes connected via causal relationships from this node.
    for rel in &project.timeline.relationships {
        if rel.from_node == node_id && matches!(rel.relationship_type, RelationshipType::Causal) {
            if let Ok(target) = project.timeline.node(rel.to_node) {
                if !target.locked && has_content(target) && !ids.contains(&rel.to_node) {
                    ids.push(rel.to_node);
                }
            }
        }
    }

    ids
}

fn has_content(node: &StoryNode) -> bool {
    !node.content.content.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Template;
    use crate::timeline::node::StoryLevel;

    #[test]
    fn downstream_excludes_locked_nodes() {
        let mut project = Template::MultiCam.build_project("Test");

        // Get scene nodes and lock the second one.
        let scenes: Vec<NodeId> = project
            .timeline
            .nodes_at_level(StoryLevel::Scene)
            .iter()
            .map(|n| n.id)
            .collect();

        if scenes.len() > 1 {
            project.timeline.node_mut(scenes[1]).unwrap().locked = true;
        }

        let downstream = downstream_node_ids(&project, scenes[0]);
        for id in &downstream {
            let node = project.timeline.node(*id).unwrap();
            assert!(!node.locked);
        }
    }

    #[test]
    fn build_edit_context_returns_content() {
        let mut project = Template::MultiCam.build_project("Test");

        let scenes = project.timeline.nodes_at_level(StoryLevel::Scene);
        if scenes.is_empty() {
            return; // Template may not have scenes yet.
        }
        let node_id = scenes[0].id;

        let node = project.timeline.node_mut(node_id).unwrap();
        node.content.content = "Edited script.".into();

        let ctx = build_edit_context(&project, node_id).unwrap();
        assert!(ctx.previous_script.is_empty());
        assert_eq!(ctx.new_script, "Edited script.");
    }
}
