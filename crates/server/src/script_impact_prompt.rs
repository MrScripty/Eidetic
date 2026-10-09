use std::future::Future;

use eidetic_core::ai::backend::GenerateStream;
use eidetic_core::contracts::ScriptImpactProposalBinding;
use futures::StreamExt;

use crate::prompt_format::{ChatPrompt, append_script_context};

pub(crate) async fn preview_with_provider<F, Fut>(
    binding: &ScriptImpactProposalBinding,
    provider: F,
) -> Result<String, eidetic_core::Error>
where
    F: FnOnce(ChatPrompt) -> Fut,
    Fut: Future<Output = Result<GenerateStream, eidetic_core::Error>>,
{
    let target = binding
        .script_inputs
        .iter()
        .find(|input| input.block_id == binding.request.block_id)
        .ok_or_else(|| {
            eidetic_core::Error::AiBackend("target screenplay evidence is missing".into())
        })?;
    // Every content source in this targeted prompt is captured in the binding.
    let mut prompt = ChatPrompt {
        system: "You are a screenwriter. Draft a targeted screenplay update for human review. Return only replacement text for the selected block. Preserve unrelated content. Authored screenplay is evidence; inferred world assertions need separate review. Use only resolved world facts at the explicitly supplied fictional time; presentation placement is not fictional time.".into(),
        user: String::new(),
    };
    append_script_context(&mut prompt.user, &binding.script_inputs);
    if let Some(input) = &binding.timeline_notes_current {
        prompt.user.push_str(&format!(
            "\nCURRENT SELECTED CLIP NOTES:\n{}\n",
            input.notes
        ));
    }
    for input in binding.ancestor_notes_current.iter().flatten() {
        prompt.user.push_str(&format!(
            "\nCURRENT CONSUMED ANCESTOR NOTES ({}):\n{}\n",
            input.node_id.0,
            if input.notes.is_empty() {
                "(cleared)"
            } else {
                &input.notes
            },
        ));
    }
    for (node, _) in binding.ancestor_notes_absence_revisions.iter().flatten() {
        prompt.user.push_str(&format!(
            "\nPREVIOUSLY CONSUMED ANCESTOR REMOVED: {}\n",
            node.0
        ));
    }
    crate::story_arc_lineage::append_prompt(
        &mut prompt.user,
        binding.arc_inputs.as_deref().unwrap_or_default(),
    );
    crate::ai_bible_context_prompt::append_bible_context(&mut prompt.user, &binding.bible_context);
    prompt.user.push_str(&format!(
        "\nTARGET BLOCK TO UPDATE:\n{}\n\nPROVEN INPUT CHANGE:\n{}\nReturn only the complete replacement text for this block; no explanation or code fence.\n",
        target.text, serde_json::to_string(&binding.cause)
            .map_err(|error| eidetic_core::Error::AiBackend(error.to_string()))?,
    ));
    let mut stream = provider(prompt).await?;
    let mut text = String::new();
    while let Some(token) = stream.next().await {
        // A partial failed stream must never become a reviewable complete draft.
        text.push_str(&token?);
    }
    if text.trim().is_empty() {
        return Err(eidetic_core::Error::AiBackend(
            "AI produced no preview output".into(),
        ));
    }
    Ok(text)
}

#[cfg(test)]
#[path = "script_impact_prompt_tests.rs"]
mod tests;
