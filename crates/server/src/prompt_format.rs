use uuid::Uuid;

use eidetic_core::ai::backend::{EditContext, GenerateRequest, PlanBeatsRequest};
use eidetic_core::story::bible::ResolvedEntity;
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

    // Story bible — entities directly referenced by this beat (full detail).
    if !request.bible_context.referenced_entities.is_empty() {
        system.push_str("\nSTORY BIBLE — Key entities in this scene:\n");
        for entity in &request.bible_context.referenced_entities {
            if let Some(ref full) = entity.full_text {
                system.push_str(full);
                system.push('\n');
            } else {
                system.push_str(&format!("- {}\n", entity.compact_text));
            }
        }
    }

    // Other active entities (compact, for awareness).
    if !request.bible_context.nearby_entities.is_empty() {
        system.push_str("\nOTHER ACTIVE ENTITIES (for awareness):\n");
        for entity in &request.bible_context.nearby_entities {
            system.push_str(&format!("- {}\n", entity.compact_text));
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

    // Sub-beat context: parent scene notes and sibling beat structure.
    if let Some(ref parent_notes) = request.parent_scene_notes {
        user.push_str("SCENE CONTEXT (parent scene this beat belongs to):\n");
        user.push_str(parent_notes);
        user.push_str("\n\n");
    }

    if !request.sibling_beat_outlines.is_empty() {
        user.push_str("SCENE BEAT STRUCTURE (all beats in this scene — you are writing one of these):\n");
        for (name, beat_type, outline) in &request.sibling_beat_outlines {
            let marker = if *name == request.beat_clip.name { " ← YOU ARE HERE" } else { "" };
            user.push_str(&format!("- {} ({}): {}{}\n", name, beat_type, outline, marker));
        }
        user.push_str("\nWrite ONLY the beat marked above. Stay focused on this single beat.\n\n");
    }

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

    // Cross-track continuity recaps — distilled scene state summaries.
    if !request.surrounding_context.preceding_recaps.is_empty() {
        user.push_str(
            "CONTINUITY CONTEXT — Scene recaps from preceding clips across all storylines.\n\
             THESE ARE ESTABLISHED FACTS. Your output must not contradict them:\n\n",
        );
        for entry in &request.surrounding_context.preceding_recaps {
            user.push_str(&format!(
                "--- {} / {} ---\n{}\n\n",
                entry.arc_name, entry.clip_name, entry.recap,
            ));
        }
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

    // RAG reference material.
    if !request.rag_context.is_empty() {
        user.push_str("REFERENCE MATERIAL (use to inform tone, world details, and character voices):\n");
        for chunk in &request.rag_context {
            user.push_str(&format!(
                "--- {} (relevance: {:.0}%) ---\n{}\n\n",
                chunk.source,
                chunk.relevance_score * 100.0,
                chunk.content,
            ));
        }
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

/// Build a chat prompt for entity extraction from a generated script.
pub(crate) fn build_extraction_prompt(
    script: &str,
    existing_entities: &[ResolvedEntity],
    time_ms: u64,
) -> ChatPrompt {
    let system = String::from(
        "You are a story analyst for a 30-minute TV episode. \
         Given a screenplay beat, identify ALL characters, locations, props, themes, \
         and events present in the scene, along with any character development points.\n\n\
         SCREENPLAY PARSING HINTS:\n\
         - Scene headings (INT./EXT. lines) indicate locations.\n\
         - Character names in ALLCAPS above dialogue lines indicate speaking characters.\n\
         - Characters mentioned in action/description lines are also present.\n\
         - Characters referred to by pronoun or nickname should be identified by their full name when possible.\n\n\
         RULES:\n\
         - `entities_present` MUST list the name of EVERY character, location, and notable entity \
         that appears in or is mentioned in the scene — both existing and new. This is the most \
         important field. Do not omit anyone.\n\
         - `new_entities` MUST contain EVERY entity from the scene that is NOT in the KNOWN ENTITIES list. \
         If a character speaks, appears, or is mentioned and they are not in the known list, add them. \
         Even minor or unnamed characters (e.g. \"Bartender\", \"Guard #1\") should be included.\n\
         - Err on the side of including too many new entities rather than too few.\n\
         - Snapshots are for meaningful development points (introductions, revelations, transformations).\n\
         - Keep taglines under 15 words.\n\
         - Return ONLY valid JSON, no commentary.",
    );

    let mut user = String::from("Analyze this screenplay beat and extract entities and development points.\n\n");

    user.push_str(&format!("TIMELINE POSITION: {}ms\n\n", time_ms));

    user.push_str("SCRIPT:\n");
    user.push_str(script);
    user.push_str("\n\n");

    if !existing_entities.is_empty() {
        user.push_str("KNOWN ENTITIES (do NOT re-suggest these as new, but DO include them in entities_present if they appear):\n");
        for entity in existing_entities {
            user.push_str(&format!("- {} [{}]\n", entity.name, entity.category));
        }
        user.push('\n');
    }

    user.push_str(
        "Respond with JSON in this exact format:\n\
         ```json\n\
         {\n\
           \"entities_present\": [\"<name of every entity appearing in this scene, existing or new>\"],\n\
           \"new_entities\": [\n\
             {\n\
               \"name\": \"<entity name — MUST NOT be in known entities list>\",\n\
               \"category\": \"<one of: Character, Location, Prop, Theme, Event>\",\n\
               \"tagline\": \"<brief description, under 15 words>\",\n\
               \"description\": \"<fuller description>\"\n\
             }\n\
           ],\n\
           \"snapshot_suggestions\": [\n\
             {\n\
               \"entity_name\": \"<name of existing or new entity>\",\n\
               \"description\": \"<what changed or what we learn>\",\n\
               \"emotional_state\": \"<optional: character's feeling>\",\n\
               \"audience_knowledge\": \"<optional: what audience now knows>\",\n\
               \"location\": \"<optional: where this entity is, e.g. INT. CABIN - MORNING>\"\n\
             }\n\
           ]\n\
         }\n\
         ```\n\
         IMPORTANT: Every character, location, prop, theme, or event that appears in the script \
         and is NOT in the KNOWN ENTITIES list MUST appear in new_entities. Do not skip any.\n\
         Return `{\"entities_present\": [], \"new_entities\": [], \"snapshot_suggestions\": []}` if no entities found.",
    );

    ChatPrompt { system, user }
}

/// Build a chat prompt to generate a compact scene recap from a script.
///
/// The recap captures scene-end state, established facts, and narrative
/// events in ~100-200 tokens of structured text. If a preceding recap
/// is provided, the generated recap carries forward still-relevant facts
/// (rolling summary behavior).
pub(crate) fn build_recap_prompt(
    script: &str,
    preceding_recap: Option<&str>,
) -> ChatPrompt {
    let system = String::from(
        "You are a script continuity analyst. Given a screenplay scene, produce a \
         compact structured recap that captures the scene's end state. This recap \
         will be used as context for writing subsequent scenes to maintain continuity.\n\n\
         FORMAT (use this exact structure):\n\
         SCENE END STATE:\n\
         - Location: [INT/EXT. LOCATION - TIME]\n\
         - Characters present: [list]\n\
         - [Brief physical/emotional state of each character]\n\n\
         KEY ESTABLISHED FACTS:\n\
         - [Relationships, names, details that must remain consistent]\n\n\
         WHAT JUST HAPPENED:\n\
         - [3-5 bullet points summarizing key events in order]\n\n\
         RULES:\n\
         - Keep the total recap under 200 tokens.\n\
         - Focus on FACTS that downstream scenes must respect.\n\
         - Include character locations and physical states at scene end.\n\
         - Carry forward any still-relevant facts from the preceding recap.\n\
         - Do NOT include analysis or suggestions — only factual statements.\n\
         - Return ONLY the recap text, no commentary.",
    );

    let mut user = String::from("Generate a scene recap for this screenplay beat:\n\n");
    user.push_str("SCRIPT:\n");
    user.push_str(script);
    user.push('\n');

    if let Some(prev) = preceding_recap {
        user.push_str("\nPRECEDING SCENE RECAP (carry forward still-relevant facts):\n");
        user.push_str(prev);
        user.push('\n');
    }

    user.push_str(
        "\nProduce the scene recap now. Use the exact format specified. \
         Be concise — aim for 100-150 tokens.",
    );

    ChatPrompt { system, user }
}

/// Build a chat prompt for beat planning — decomposing a scene clip into beats.
pub(crate) fn build_beat_plan_prompt(request: &PlanBeatsRequest) -> ChatPrompt {
    let mut system = String::from(
        "You are a story structure analyst for a 30-minute TV episode. \
         Given a scene description, break it down into individual narrative beats.\n\n\
         BEAT TYPES (choose the most appropriate for each):\n\
         - Setup: Establishes setting, characters, or situation\n\
         - Complication: Introduces a problem or obstacle\n\
         - Escalation: Raises stakes or tension\n\
         - Climax: Peak moment of conflict or revelation\n\
         - Resolution: Resolves the immediate conflict\n\
         - Payoff: Delivers on earlier setup\n\
         - Callback: References earlier material\n\n\
         RULES:\n\
         - Propose 3-7 beats depending on scene complexity.\n\
         - Each beat should be a single dramatic moment or shift.\n\
         - Outlines should be 1-2 sentences describing what happens.\n\
         - Weights represent relative duration (1.0 = normal, 0.5 = brief, 2.0 = extended).\n\
         - Beats should flow naturally from one to the next.\n\
         - Return ONLY valid JSON, no commentary.\n",
    );

    // Story bible — entities directly referenced by this scene (full detail).
    if !request.bible_context.referenced_entities.is_empty() {
        system.push_str("\nSTORY BIBLE — Key entities in this scene:\n");
        for entity in &request.bible_context.referenced_entities {
            if let Some(ref full) = entity.full_text {
                system.push_str(full);
                system.push('\n');
            } else {
                system.push_str(&format!("- {}\n", entity.compact_text));
            }
        }
    }

    // Other active entities (compact, for awareness).
    if !request.bible_context.nearby_entities.is_empty() {
        system.push_str("\nOTHER ACTIVE ENTITIES (characters, locations, props available):\n");
        for entity in &request.bible_context.nearby_entities {
            system.push_str(&format!("- {}\n", entity.compact_text));
        }
    }

    system.push_str(
        "\nInclude which characters, locations, and props are involved in each beat's outline. \
         Reference entity names from the story bible when applicable.\n",
    );

    let mut user = String::from("Break this scene into individual narrative beats:\n\n");

    // Arc context.
    user.push_str(&format!(
        "STORY ARC: {} ({:?})",
        request.arc.name, request.arc.arc_type,
    ));
    if !request.arc.description.is_empty() {
        user.push_str(&format!(" — {}", request.arc.description));
    }
    user.push('\n');

    // Scene info.
    user.push_str(&format!(
        "SCENE: {} ({:?})\n",
        request.beat_clip.name, request.beat_clip.beat_type,
    ));

    // Scene notes.
    user.push_str("SCENE NOTES:\n");
    user.push_str(&request.beat_clip.content.beat_notes);
    user.push_str("\n\n");

    // Duration context.
    let duration_ms = request.beat_clip.time_range.duration_ms();
    let duration_sec = duration_ms / 1000;
    user.push_str(&format!(
        "SCENE DURATION: {} seconds ({} minutes {} seconds)\n\n",
        duration_sec,
        duration_sec / 60,
        duration_sec % 60,
    ));

    // Cross-track continuity.
    if !request.surrounding_context.preceding_recaps.is_empty() {
        user.push_str("CONTINUITY CONTEXT:\n");
        for entry in &request.surrounding_context.preceding_recaps {
            user.push_str(&format!(
                "- {} / {}: {}\n",
                entry.arc_name, entry.clip_name, entry.recap,
            ));
        }
        user.push('\n');
    }

    user.push_str(
        "Respond with a JSON array of beats:\n\
         ```json\n\
         [\n\
           {\n\
             \"name\": \"<short descriptive name>\",\n\
             \"beat_type\": \"<one of: Setup, Complication, Escalation, Climax, Resolution, Payoff, Callback>\",\n\
             \"outline\": \"<1-2 sentence description of what happens in this beat>\",\n\
             \"weight\": <relative duration, e.g. 1.0>\n\
           }\n\
         ]\n\
         ```",
    );

    ChatPrompt { system, user }
}
