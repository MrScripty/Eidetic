use std::pin::Pin;

use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::story::arc::StoryArc;
use crate::story::bible::{BibleContext, ExtractionResult, ResolvedEntity};
use crate::timeline::clip::BeatClip;

/// Token-by-token stream of generated script text.
pub type GenerateStream = Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>;

/// Backend-agnostic interface for AI generation.
///
/// Defined in `eidetic-core` so the library can reference it in orchestration
/// logic. Concrete implementations (llama.cpp, OpenRouter) live in `eidetic-server`.
pub trait AiBackend: Send + Sync {
    /// Generate script text for a beat clip given its notes and context.
    fn generate(
        &self,
        request: GenerateRequest,
    ) -> impl std::future::Future<Output = Result<GenerateStream, Error>> + Send;

    /// React to a user edit and suggest consistency updates to other beats.
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

/// Everything the AI needs to generate script for a single beat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// The beat to generate for.
    pub beat_clip: BeatClip,
    /// Which arc this beat belongs to.
    pub arc: StoryArc,
    /// Beats on other arcs at the same time position (for scene weaving).
    pub overlapping_beats: Vec<(BeatClip, StoryArc)>,
    /// Story bible entities resolved at this beat's time position.
    pub bible_context: BibleContext,
    /// Scripts from adjacent beats (preceding / following).
    pub surrounding_context: SurroundingContext,
    /// Target screen time for this beat (milliseconds).
    pub time_budget_ms: u64,
    /// Text the user wrote that must appear verbatim.
    pub user_written_anchors: Vec<String>,
    pub style_notes: Option<String>,
    pub rag_context: Vec<RagChunk>,
}

/// Adjacent beat scripts for context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurroundingContext {
    pub preceding_scripts: Vec<String>,
    pub following_scripts: Vec<String>,
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
    pub beat_clip: BeatClip,
    pub previous_script: String,
    pub new_script: String,
    pub surrounding_context: SurroundingContext,
}

/// A suggested update to a downstream beat after a user edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyUpdate {
    pub target_clip_id: crate::timeline::clip::ClipId,
    pub original_text: String,
    pub suggested_text: String,
    pub reason: String,
}
