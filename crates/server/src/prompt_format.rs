use uuid::Uuid;

use eidetic_core::ai::backend::{EditContext, GenerateRequest};
use eidetic_core::timeline::timing::TimeRange;

/// A structured chat prompt ready for serialization to any backend API.
pub(crate) struct ChatPrompt {
    pub system: String,
    pub user: String,
}

/// Build a chat prompt from a `GenerateRequest`.
pub(crate) fn build_chat_prompt(request: &GenerateRequest) -> ChatPrompt {
    ChatPrompt {
        system: build_system_message(request),
        user: build_user_message(request),
    }
}

fn build_system_message(request: &GenerateRequest) -> String {
    let mut system = String::from(
        "You are an experienced TV screenwriter writing a 30-minute comedy/drama episode. \
         Write in standard screenplay format.\n\n\
         FORMAT RULES:\n\
         - Scene headings: INT. or EXT. followed by LOCATION - TIME OF DAY (in ALL CAPS)\n\
         - Action lines: present tense, vivid but concise\n\
         - Character names: ALL CAPS, centered above their dialogue\n\
         - Parentheticals: in (parentheses) below character name, only when absolutely necessary\n\
         - Dialogue: natural, character-specific speech patterns\n\
         - Transitions: CUT TO:, SMASH CUT TO:, etc. (use sparingly)\n",
    );

    // Character voices.
    if !request.characters.is_empty() {
        system.push_str("\nCHARACTER VOICES:\n");
        for character in &request.characters {
            system.push_str(&format!("- {}", character.name));
            if !character.description.is_empty() {
                system.push_str(&format!(": {}", character.description));
            }
            if !character.voice_notes.is_empty() {
                system.push_str(&format!(" — Voice: {}", character.voice_notes));
            }
            system.push('\n');
        }
    }

    // Page budget.
    let time_range = TimeRange {
        start_ms: 0,
        end_ms: request.time_budget_ms,
    };
    system.push_str(&format!(
        "\nPAGE BUDGET:\nThis beat should be {}. \
         Do not significantly exceed or fall short of this target.\n",
        time_range.page_budget_instruction()
    ));

    system
}

fn build_user_message(request: &GenerateRequest) -> String {
    let mut user = String::from("Write the screenplay for the following beat:\n\n");

    // Arc context.
    user.push_str(&format!(
        "STORY ARC: {} ({:?})",
        request.arc.name, request.arc.arc_type,
    ));
    if !request.arc.description.is_empty() {
        user.push_str(&format!(" — {}", request.arc.description));
    }
    user.push('\n');

    // Beat info.
    user.push_str(&format!(
        "BEAT: {} ({:?})\n",
        request.beat_clip.name, request.beat_clip.beat_type,
    ));

    // Beat notes — the primary content.
    user.push_str("BEAT NOTES:\n");
    user.push_str(&request.beat_clip.content.beat_notes);
    user.push_str("\n\n");

    // Scene weaving for overlapping beats.
    if !request.overlapping_beats.is_empty() {
        user.push_str(
            "SCENE WEAVING — This scene also advances these storylines simultaneously:\n",
        );
        for (clip, arc) in &request.overlapping_beats {
            user.push_str(&format!("- {} ({}): ", arc.name, clip.name));
            if clip.content.beat_notes.is_empty() {
                user.push_str("[no notes yet]");
            } else {
                user.push_str(&clip.content.beat_notes);
            }
            user.push('\n');
        }
        user.push_str(
            "Weave all storylines into a single unified scene. \
             Characters from different arcs should interact naturally.\n\n",
        );
    }

    // Surrounding scripts for continuity.
    if !request.surrounding_context.preceding_scripts.is_empty() {
        user.push_str("PRECEDING SCRIPT (for continuity):\n");
        for script in &request.surrounding_context.preceding_scripts {
            user.push_str(script);
            user.push_str("\n---\n");
        }
        user.push('\n');
    }

    if !request.surrounding_context.following_scripts.is_empty() {
        user.push_str(
            "FOLLOWING SCRIPT (for continuity — your output should lead naturally into this):\n",
        );
        for script in &request.surrounding_context.following_scripts {
            user.push_str(script);
            user.push_str("\n---\n");
        }
        user.push('\n');
    }

    // User-written anchors.
    if !request.user_written_anchors.is_empty() {
        user.push_str("USER-WRITTEN ANCHORS (must appear verbatim in your output):\n");
        for anchor in &request.user_written_anchors {
            user.push_str(&format!(">>> {anchor}\n"));
        }
        user.push('\n');
    }

    // Style notes.
    if let Some(notes) = &request.style_notes {
        user.push_str(&format!("STYLE NOTES: {notes}\n\n"));
    }

    user.push_str("Write ONLY the screenplay text for this beat. Do not include metadata, comments, or explanations.");

    user
}

/// Build a chat prompt for the consistency reaction pipeline.
///
/// `downstream_beats` is a slice of `(clip_id, beat_name, current_script)` tuples.
pub(crate) fn build_consistency_prompt(
    edit_context: &EditContext,
    downstream_beats: &[(Uuid, String, String)],
) -> ChatPrompt {
    let system = String::from(
        "You are a script consistency analyst for a 30-minute TV episode. \
         Given an edit to a screenplay beat, identify necessary changes to \
         downstream beats to maintain continuity.\n\n\
         RULES:\n\
         - Only suggest changes that are strictly necessary for consistency \
           (character names, plot references, continuity details).\n\
         - Do not rewrite scenes for style — only fix factual/continuity breaks.\n\
         - If no changes are needed, return an empty JSON array.\n\
         - Return ONLY valid JSON, no commentary.",
    );

    let mut user = String::from("A screenplay beat was edited. Analyze whether downstream beats need updates.\n\n");

    user.push_str(&format!(
        "EDITED BEAT: {}\n\nBEFORE:\n{}\n\nAFTER:\n{}\n\n",
        edit_context.beat_clip.name,
        edit_context.previous_script,
        edit_context.new_script,
    ));

    if !downstream_beats.is_empty() {
        user.push_str("DOWNSTREAM BEATS TO CHECK:\n\n");
        for (id, name, script) in downstream_beats {
            user.push_str(&format!("--- Beat: {} (ID: {}) ---\n{}\n\n", name, id, script));
        }
    }

    user.push_str(
        "Respond with a JSON array of suggested changes:\n\
         ```json\n\
         [\n\
           {\n\
             \"target_clip_id\": \"<uuid of the downstream beat>\",\n\
             \"original_text\": \"<exact snippet from the downstream beat to replace>\",\n\
             \"suggested_text\": \"<replacement text>\",\n\
             \"reason\": \"<brief explanation of why this change is needed>\"\n\
           }\n\
         ]\n\
         ```\n\
         Return `[]` if no changes are needed.",
    );

    ChatPrompt { system, user }
}
