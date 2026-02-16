use crate::ai::backend::{GenerateChildrenRequest, GenerateRequest};
use crate::error::{Error, Result};
use crate::project::Project;
use crate::story::bible::{gather_bible_context, BibleContext, ResolvedEntity};
use crate::timeline::node::NodeId;

use super::helpers::{gather_recap_context, gather_surrounding_context};

/// Build a [`GenerateRequest`] for a specific story node from the project state.
///
/// Gathers:
/// - The target node and its tagged arcs
/// - Ancestor chain (parent, grandparent, etc.)
/// - Sibling nodes at the same level
/// - Story bible entities resolved at this node's time position
/// - Surrounding content from sibling nodes
pub fn build_generate_request(project: &Project, node_id: NodeId) -> Result<GenerateRequest> {
    let timeline = &project.timeline;

    let target_node = timeline.node(node_id)?.clone();

    // Gather tagged arcs for this node.
    let arc_ids = timeline.arcs_for_node(node_id);
    let tagged_arcs: Vec<_> = project
        .arcs
        .iter()
        .filter(|a| arc_ids.contains(&a.id))
        .cloned()
        .collect();

    // Gather ancestor chain (parent, grandparent, etc.).
    let ancestor_chain: Vec<_> = timeline.ancestors_of(node_id).into_iter().cloned().collect();

    // Gather siblings at the same level.
    let siblings: Vec<_> = timeline.siblings_of(node_id).into_iter().cloned().collect();

    // Gather surrounding context from siblings.
    let mut surrounding_context = gather_surrounding_context(timeline, node_id);

    // Gather cross-node recaps for continuity.
    surrounding_context.preceding_recaps =
        gather_recap_context(&project.timeline, &project.arcs, node_id);

    // Gather bible context resolved at the node's midpoint time.
    let node_mid_ms = target_node.time_range.start_ms
        + target_node.time_range.duration_ms() / 2;
    let bible_context = gather_bible_context(&project.bible, node_id, node_mid_ms);

    let time_budget_ms = target_node.time_range.duration_ms();

    Ok(GenerateRequest {
        target_node,
        tagged_arcs,
        ancestor_chain,
        siblings,
        bible_context,
        surrounding_context,
        time_budget_ms,
        user_written_anchors: vec![],
        style_notes: None,
        rag_context: vec![],
    })
}

/// Build a [`GenerateChildrenRequest`] for decomposing a parent node into children.
///
/// Gathers the same context as `build_generate_request` but focused on the
/// parent node. The AI will use this to generate child proposals.
pub fn build_generate_children_request(
    project: &Project,
    parent_node_id: NodeId,
) -> Result<GenerateChildrenRequest> {
    let timeline = &project.timeline;

    let parent_node = timeline.node(parent_node_id)?.clone();

    let target_child_level = parent_node.level.child_level().ok_or_else(|| {
        Error::InvalidHierarchy(format!(
            "{} nodes cannot have children",
            parent_node.level
        ))
    })?;

    let arc_ids = timeline.arcs_for_node(parent_node_id);
    let tagged_arcs: Vec<_> = project
        .arcs
        .iter()
        .filter(|a| arc_ids.contains(&a.id))
        .cloned()
        .collect();

    let mut surrounding_context = gather_surrounding_context(timeline, parent_node_id);
    surrounding_context.preceding_recaps =
        gather_recap_context(timeline, &project.arcs, parent_node_id);

    let node_mid_ms =
        parent_node.time_range.start_ms + parent_node.time_range.duration_ms() / 2;

    // For child planning, include ALL entities with full detail.
    let bible_context = BibleContext {
        referenced_entities: project
            .bible
            .entities
            .iter()
            .map(|e| ResolvedEntity {
                entity_id: e.id,
                name: e.name.clone(),
                category: e.category.clone(),
                compact_text: e.to_prompt_text(node_mid_ms),
                full_text: Some(e.to_full_prompt_text(node_mid_ms)),
            })
            .collect(),
        nearby_entities: Vec::new(),
    };

    // Include episode structure for Premise → Act decomposition.
    let episode_structure = if parent_node.level == crate::timeline::node::StoryLevel::Premise {
        Some(timeline.structure.clone())
    } else {
        None
    };

    Ok(GenerateChildrenRequest {
        parent_node,
        target_child_level,
        tagged_arcs,
        bible_context,
        surrounding_context,
        episode_structure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::node::StoryLevel;
    use crate::Template;

    #[test]
    fn build_request_from_template() {
        let project = Template::MultiCam.build_project("Test");
        let timeline = &project.timeline;

        // Get the first scene node.
        let scenes = timeline.nodes_at_level(StoryLevel::Scene);
        assert!(!scenes.is_empty(), "template should have scene nodes");

        let first_scene_id = scenes[0].id;
        let req = build_generate_request(&project, first_scene_id).unwrap();

        assert_eq!(req.target_node.id, first_scene_id);
        assert_eq!(req.time_budget_ms, req.target_node.time_range.duration_ms());
    }

    #[test]
    fn build_request_node_not_found() {
        let project = Template::MultiCam.build_project("Test");
        let bogus_id = NodeId::new();
        let result = build_generate_request(&project, bogus_id);
        assert!(result.is_err());
    }
}
