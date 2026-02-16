use std::pin::Pin;

use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::story::arc::StoryArc;
use crate::story::bible::{BibleContext, ExtractionResult, ResolvedEntity};
use crate::timeline::node::{BeatType, NodeId, StoryLevel, StoryNode};
use crate::timeline::structure::EpisodeStructure;

/// Token-by-token stream of generated text.
pub type GenerateStream = Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>;

/// Backend-agnostic interface for AI generation.
///
/// Defined in `eidetic-core` so the library can reference it in orchestration
/// logic. Concrete implementations (Ollama, OpenRouter) live in `eidetic-server`.
pub trait AiBackend: Send + Sync {
    /// Generate content for a story node given its notes and context.
    fn generate(
        &self,
        request: GenerateRequest,
    ) -> impl std::future::Future<Output = Result<GenerateStream, Error>> + Send;

    /// React to a user edit and suggest consistency updates to other nodes.
    fn react_to_edit(
        &self,
        edit: EditContext,
    ) -> impl std::future::Future<Output = Result<Vec<ConsistencyUpdate>, Error>> + Send;

    /// Summarize a section for use as context elsewhere.
    fn summarize(
        &self,
        text: &str,
    ) -> impl std::future::Future<Output = Result<String, Error>> + Send;

    /// Extract entities and development points from a generated script.
    fn extract_entities(
        &self,
        script: &str,
        existing_entities: &[ResolvedEntity],
        time_ms: u64,
    ) -> impl std::future::Future<Output = Result<ExtractionResult, Error>> + Send;
}

/// Everything the AI needs to generate content for a single story node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// The node to generate content for (at any level).
    pub target_node: StoryNode,
    /// Arcs tagged on this node.
    pub tagged_arcs: Vec<StoryArc>,
    /// The target's ancestors up to Act level (parent first, root last).
    #[serde(default)]
    pub ancestor_chain: Vec<StoryNode>,
    /// Sibling nodes at the same level (for structural context).
    #[serde(default)]
    pub siblings: Vec<StoryNode>,
    /// Story bible entities resolved at this node's time position.
    pub bible_context: BibleContext,
    /// Scripts/content from adjacent nodes.
    pub surrounding_context: SurroundingContext,
    /// Target screen time for this node (milliseconds).
    pub time_budget_ms: u64,
    /// Text the user wrote that must appear verbatim.
    pub user_written_anchors: Vec<String>,
    pub style_notes: Option<String>,
    pub rag_context: Vec<RagChunk>,
}

/// Adjacent node content for context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurroundingContext {
    pub preceding_scripts: Vec<String>,
    pub following_scripts: Vec<String>,
    /// Recaps from nodes that temporally precede the target node's start time.
    #[serde(default)]
    pub preceding_recaps: Vec<RecapEntry>,
}

/// A scene recap from a preceding node, including source identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapEntry {
    pub arc_name: String,
    pub node_name: String,
    /// End time of the source node (ms), for ordering.
    pub end_time_ms: u64,
    /// The recap text.
    pub recap: String,
}

/// A chunk of reference material retrieved via RAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub source: String,
    pub content: String,
    pub relevance_score: f32,
}

/// Context for the edit-reaction pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditContext {
    pub node: StoryNode,
    pub previous_script: String,
    pub new_script: String,
    pub surrounding_context: SurroundingContext,
}

/// A suggested update to a downstream node after a user edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyUpdate {
    pub target_node_id: NodeId,
    pub original_text: String,
    pub suggested_text: String,
    pub reason: String,
}

/// A single proposed child in a decomposition plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildProposal {
    /// Name for this child node.
    pub name: String,
    /// The hierarchy level of this child (set server-side, not by AI).
    #[serde(default)]
    pub level: Option<StoryLevel>,
    /// Structural beat type (only for Beat-level children).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat_type: Option<BeatType>,
    /// 1-2 sentence description of what happens.
    pub outline: String,
    /// Relative duration weight (1.0 = normal, 2.0 = twice as long).
    pub weight: f32,
    /// Characters who appear in this child.
    #[serde(default)]
    pub characters: Vec<String>,
    /// Scene heading location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Props that are involved.
    #[serde(default)]
    pub props: Vec<String>,
}

/// AI-generated plan for decomposing a parent node into children.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPlan {
    pub parent_node_id: NodeId,
    pub target_child_level: StoryLevel,
    pub children: Vec<ChildProposal>,
}

/// Everything the AI needs to plan children for a parent node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateChildrenRequest {
    /// The parent node to decompose.
    pub parent_node: StoryNode,
    /// What level of children to generate.
    pub target_child_level: StoryLevel,
    /// Arcs tagged on this node.
    pub tagged_arcs: Vec<StoryArc>,
    /// Story bible entities resolved at this node's time position.
    pub bible_context: BibleContext,
    /// Content from adjacent nodes.
    pub surrounding_context: SurroundingContext,
    /// Episode structure (act segments, commercial breaks). Included for
    /// Premise → Act decomposition so the AI knows the expected act layout.
    #[serde(default)]
    pub episode_structure: Option<EpisodeStructure>,
}

/// Everything the AI needs to infer a parent from children.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateUpwardRequest {
    /// The children to infer a parent from.
    pub children: Vec<StoryNode>,
    /// What level to generate at.
    pub target_parent_level: StoryLevel,
    /// Arcs tagged on the children.
    pub tagged_arcs: Vec<StoryArc>,
    /// Story bible context.
    pub bible_context: BibleContext,
}
