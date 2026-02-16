use uuid::Uuid;

use eidetic_core::ai::backend::{EditContext, GenerateChildrenRequest, GenerateRequest};
use eidetic_core::story::bible::ResolvedEntity;
use eidetic_core::timeline::node::StoryLevel;
use eidetic_core::timeline::structure::SegmentType;
use eidetic_core::timeline::timing::TimeRange;

/// A structured chat prompt ready for serialization to any backend API.
pub(crate) struct ChatPrompt {
    pub system: String,
    pub user: String,
}

/// Build a chat prompt from a `GenerateRequest`.
///
/// Works for any hierarchy level — adapts instructions based on the target
/// node's level (Beat → screenplay format, higher levels → structural outline).
pub(crate) fn build_chat_prompt(request: &GenerateRequest) -> ChatPrompt {
    ChatPrompt {
        system: build_system_message(request),
        user: build_user_message(request),
    }
}

fn build_system_message(request: &GenerateRequest) -> String {
    let level = request.target_node.level;

    let mut system = if level == StoryLevel::Beat {
        String::from(
            "You are an experienced TV screenwriter writing a 30-minute comedy/drama episode. \
             Write in standard screenplay format.\n\n\
             FORMAT RULES:\n\
             - Scene headings: INT. or EXT. followed by LOCATION - TIME OF DAY (in ALL CAPS)\n\
             - Action lines: present tense, vivid but concise\n\
             - Character names: ALL CAPS, centered above their dialogue\n\
             - Parentheticals: in (parentheses) below character name, only when absolutely necessary\n\
             - Dialogue: natural, character-specific speech patterns\n\
             - Transitions: CUT TO:, SMASH CUT TO:, etc. (use sparingly)\n",
        )
    } else {
        format!(
            "You are an experienced TV story editor working on a 30-minute comedy/drama episode. \
             Write a structural outline for this {} node.\n\n\
             FORMAT RULES:\n\
             - Write in clear prose, not screenplay format.\n\
             - Describe what happens narratively — key events, character dynamics, emotional beats.\n\
             - Focus on story structure and dramatic progression.\n\
             - Be specific about character actions and motivations.\n",
            level.label()
        )
    };

    // Story bible — entities directly referenced by this node (full detail).
    if !request.bible_context.referenced_entities.is_empty() {
        system.push_str("\nSTORY BIBLE — Key entities in this section:\n");
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

    // Page budget (only for Beat level).
    if level == StoryLevel::Beat {
        let time_range = TimeRange {
            start_ms: 0,
            end_ms: request.time_budget_ms,
        };
        system.push_str(&format!(
            "\nPAGE BUDGET:\nThis beat should be {}. \
             Do not significantly exceed or fall short of this target.\n",
            time_range.page_budget_instruction()
        ));
    }

    system
}

fn build_user_message(request: &GenerateRequest) -> String {
    let level = request.target_node.level;
    let level_name = level.label().to_lowercase();

    let mut user = if level == StoryLevel::Beat {
        String::from("Write the screenplay for the following beat:\n\n")
    } else {
        format!(
            "Write a structural outline for the following {}:\n\n",
            level_name
        )
    };

    // Arc context.
    if !request.tagged_arcs.is_empty() {
        user.push_str("STORY ARCS: ");
        let arc_strs: Vec<String> = request
            .tagged_arcs
            .iter()
            .map(|a| {
                let mut s = format!("{} ({:?})", a.name, a.arc_type);
                if !a.description.is_empty() {
                    s.push_str(&format!(" — {}", a.description));
                }
                s
            })
            .collect();
        user.push_str(&arc_strs.join("; "));
        user.push('\n');
    }

    // Node info.
    user.push_str(&format!(
        "{}: {}\n",
        level.label().to_uppercase(),
        request.target_node.name,
    ));
    if let Some(ref bt) = request.target_node.beat_type {
        user.push_str(&format!("BEAT TYPE: {:?}\n", bt));
    }

    // Notes — the primary content.
    user.push_str(&format!("{} NOTES:\n", level.label().to_uppercase()));
    user.push_str(&request.target_node.content.notes);
    user.push_str("\n\n");

    // Ancestor context (parent, grandparent, etc.).
    if !request.ancestor_chain.is_empty() {
        user.push_str("CONTEXT HIERARCHY:\n");
        for ancestor in &request.ancestor_chain {
            user.push_str(&format!(
                "- {} ({}): {}\n",
                ancestor.name,
                ancestor.level.label(),
                if ancestor.content.notes.is_empty() {
                    "[no notes]"
                } else {
                    &ancestor.content.notes
                },
            ));
        }
        user.push('\n');
    }

    // Sibling context (same level, same parent).
    if !request.siblings.is_empty() {
        user.push_str(&format!(
            "SIBLING {}S (other {}s at this level — you are writing one of these):\n",
            level.label().to_uppercase(),
            level_name,
        ));
        for sibling in &request.siblings {
            let marker = if sibling.id == request.target_node.id {
                " ← YOU ARE HERE"
            } else {
                ""
            };
            let text = sibling.best_text();
            let preview = if text.len() > 200 {
                format!("{}...", &text[..200])
            } else {
                text.to_string()
            };
            user.push_str(&format!("- {}: {}{}\n", sibling.name, preview, marker));
        }
        user.push_str(&format!(
            "\nWrite ONLY the {} marked above. Stay focused.\n\n",
            level_name
        ));
    }

    // Cross-node continuity recaps.
    if !request.surrounding_context.preceding_recaps.is_empty() {
        user.push_str(
            "CONTINUITY CONTEXT — Recaps from preceding nodes across all storylines.\n\
             THESE ARE ESTABLISHED FACTS. Your output must not contradict them:\n\n",
        );
        for entry in &request.surrounding_context.preceding_recaps {
            user.push_str(&format!(
                "--- {} / {} ---\n{}\n\n",
                entry.arc_name, entry.node_name, entry.recap,
            ));
        }
    }

    // Surrounding scripts for continuity.
    if !request.surrounding_context.preceding_scripts.is_empty() {
        user.push_str("PRECEDING CONTENT (for continuity):\n");
        for script in &request.surrounding_context.preceding_scripts {
            user.push_str(script);
            user.push_str("\n---\n");
        }
        user.push('\n');
    }

    if !request.surrounding_context.following_scripts.is_empty() {
        user.push_str(
            "FOLLOWING CONTENT (for continuity — your output should lead naturally into this):\n",
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
        user.push_str(
            "REFERENCE MATERIAL (use to inform tone, world details, and character voices):\n",
        );
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

    if level == StoryLevel::Beat {
        user.push_str(
            "Write ONLY the screenplay text for this beat. \
             Do not include metadata, comments, or explanations.",
        );
    } else {
        user.push_str(&format!(
            "Write ONLY the structural outline for this {}. \
             Do not include metadata, comments, or explanations.",
            level_name
        ));
    }

    user
}

/// Build a chat prompt for the consistency reaction pipeline.
///
/// `downstream_nodes` is a slice of `(node_id, node_name, current_text)` tuples.
pub(crate) fn build_consistency_prompt(
    edit_context: &EditContext,
    downstream_nodes: &[(Uuid, String, String)],
) -> ChatPrompt {
    let system = String::from(
        "You are a script consistency analyst for a 30-minute TV episode. \
         Given an edit to a story node, identify necessary changes to \
         downstream nodes to maintain continuity.\n\n\
         RULES:\n\
         - Only suggest changes that are strictly necessary for consistency \
           (character names, plot references, continuity details).\n\
         - Do not rewrite content for style — only fix factual/continuity breaks.\n\
         - If no changes are needed, return an empty JSON array.\n\
         - Return ONLY valid JSON, no commentary.",
    );

    let mut user =
        String::from("A story node was edited. Analyze whether downstream nodes need updates.\n\n");

    user.push_str(&format!(
        "EDITED NODE: {}\n\nBEFORE:\n{}\n\nAFTER:\n{}\n\n",
        edit_context.node.name, edit_context.previous_script, edit_context.new_script,
    ));

    if !downstream_nodes.is_empty() {
        user.push_str("DOWNSTREAM NODES TO CHECK:\n\n");
        for (id, name, text) in downstream_nodes {
            user.push_str(&format!(
                "--- Node: {} (ID: {}) ---\n{}\n\n",
                name, id, text
            ));
        }
    }

    user.push_str(
        "Respond with a JSON array of suggested changes:\n\
         ```json\n\
         [\n\
           {\n\
             \"target_node_id\": \"<uuid of the downstream node>\",\n\
             \"original_text\": \"<exact snippet from the downstream node to replace>\",\n\
             \"suggested_text\": \"<replacement text>\",\n\
             \"reason\": \"<brief explanation of why this change is needed>\"\n\
           }\n\
         ]\n\
         ```\n\
         Return `[]` if no changes are needed.",
    );

    ChatPrompt { system, user }
}

/// Build a chat prompt for entity extraction from generated text.
pub(crate) fn build_extraction_prompt(
    script: &str,
    existing_entities: &[ResolvedEntity],
    time_ms: u64,
) -> ChatPrompt {
    let system = String::from(
        "You are a story analyst for a 30-minute TV episode. \
         Given a screenplay beat, identify ALL characters, locations, props, themes, \
         and events present in the scene, along with any character development points.\n\n\
         CATEGORY DEFINITIONS:\n\
         - Character: A person, animal, or personified entity that acts, speaks, or is directly addressed.\n\
         - Location: The physical setting/environment, including ALL atmospheric and environmental \
           elements (weather, terrain, ambient features, background furniture, architectural elements, \
           natural features like snow, trees, fog, rain). If characters do not directly interact with \
           it, it is part of the Location, NOT a Prop.\n\
         - Prop: An object a character directly handles, picks up, uses, wears, or that has specific \
           plot significance. Must involve character agency or narrative function. Environmental/ \
           atmospheric elements are NEVER props.\n\
         - Theme: An abstract concept, motif, or recurring idea explored in the scene.\n\
         - Event: A significant plot occurrence, turning point, or backstory revelation.\n\n\
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
         - Do NOT tag environmental elements (snow, rain, fog, trees, furniture) as Props. \
         They belong to the Location.\n\
         - Snapshots are for meaningful development points (introductions, revelations, transformations).\n\
         - Keep taglines under 15 words.\n\
         - Return ONLY valid JSON, no commentary.",
    );

    let mut user =
        String::from("Analyze this screenplay beat and extract entities and development points.\n\n");

    user.push_str(&format!("TIMELINE POSITION: {}ms\n\n", time_ms));

    user.push_str("SCRIPT:\n");
    user.push_str(script);
    user.push_str("\n\n");

    if !existing_entities.is_empty() {
        user.push_str(
            "KNOWN ENTITIES (do NOT re-suggest these as new, but DO include them in entities_present if they appear):\n",
        );
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
pub(crate) fn build_recap_prompt(script: &str, preceding_recap: Option<&str>) -> ChatPrompt {
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

/// Build a chat prompt for decomposing a parent node into children.
///
/// Works for any level: Act → Sequences, Sequence → Scenes, Scene → Beats.
pub(crate) fn build_decompose_prompt(request: &GenerateChildrenRequest) -> ChatPrompt {
    let parent_level = request.parent_node.level;
    let child_level = request.target_child_level;
    let child_label = child_level.label().to_lowercase();

    let mut system = format!(
        "You are a story structure analyst for a 30-minute TV episode. \
         Given a {} description, break it down into individual {}s.\n\n",
        parent_level.label().to_lowercase(),
        child_label,
    );

    if child_level == StoryLevel::Beat {
        system.push_str(
            "BEAT TYPES (choose the most appropriate for each):\n\
             - Setup: Establishes setting, characters, or situation\n\
             - Complication: Introduces a problem or obstacle\n\
             - Escalation: Raises stakes or tension\n\
             - Climax: Peak moment of conflict or revelation\n\
             - Resolution: Resolves the immediate conflict\n\
             - Payoff: Delivers on earlier setup\n\
             - Callback: References earlier material\n\n",
        );
    }

    // Premise → Acts: provide the episode's act structure.
    if parent_level == StoryLevel::Premise {
        if let Some(ref structure) = request.episode_structure {
            system.push_str("EPISODE STRUCTURE:\n");
            system.push_str("This is a TV episode. You MUST generate one act for each structural \
                             segment below. Each act's name should match the segment label. \
                             Use the durations as weight guidance.\n\n");
            for seg in &structure.segments {
                let kind = match seg.segment_type {
                    SegmentType::ColdOpen => "Cold Open",
                    SegmentType::MainTitles => continue, // skip titles
                    SegmentType::Act => "Act",
                    SegmentType::CommercialBreak => continue, // skip breaks
                    SegmentType::Tag => "Tag",
                };
                let dur_sec = seg.time_range.duration_ms() / 1000;
                system.push_str(&format!(
                    "- {} \"{}\" — {} min {} sec\n",
                    kind,
                    seg.label,
                    dur_sec / 60,
                    dur_sec % 60,
                ));
            }
            system.push('\n');
        } else {
            system.push_str(
                "EPISODE STRUCTURE:\n\
                 This is a standard 30-minute TV comedy (~22 min content). You MUST generate \
                 at least these acts:\n\
                 - Cold Open (~2 min) — hook the audience\n\
                 - Act One (~7 min) — establish the premise and central conflict\n\
                 - Act Two (~7 min) — complications and escalation\n\
                 - Act Three (~5 min) — climax and resolution\n\
                 - Tag (~30 sec) — final joke or button\n\n",
            );
        }
    }

    system.push_str(&format!(
        "RULES:\n\
         - Propose 3-7 {}s depending on complexity.\n\
         - Each {} should be a coherent unit of story.\n\
         - Outlines should be 1-2 sentences describing what happens.\n\
         - Weights represent relative duration (1.0 = normal, 0.5 = brief, 2.0 = extended).\n\
         - {}s should flow naturally from one to the next.\n\
         - Return ONLY valid JSON, no commentary.\n",
        child_label, child_label, child_label,
    ));

    // Story bible context.
    if !request.bible_context.referenced_entities.is_empty() {
        system.push_str("\nSTORY BIBLE — Key entities:\n");
        for entity in &request.bible_context.referenced_entities {
            if let Some(ref full) = entity.full_text {
                system.push_str(full);
                system.push('\n');
            } else {
                system.push_str(&format!("- {}\n", entity.compact_text));
            }
        }
    }

    if !request.bible_context.nearby_entities.is_empty() {
        system.push_str("\nOTHER ACTIVE ENTITIES:\n");
        for entity in &request.bible_context.nearby_entities {
            system.push_str(&format!("- {}\n", entity.compact_text));
        }
    }

    if child_level == StoryLevel::Beat || child_level == StoryLevel::Scene {
        system.push_str(
            "\nFor each child, explicitly list the characters, location, and props involved.\n",
        );
    }

    let mut user = format!(
        "Break this {} into individual {}s:\n\n",
        parent_level.label().to_lowercase(),
        child_label,
    );

    // Arc context.
    if !request.tagged_arcs.is_empty() {
        user.push_str("STORY ARCS: ");
        let arc_strs: Vec<String> = request
            .tagged_arcs
            .iter()
            .map(|a| {
                let mut s = format!("{} ({:?})", a.name, a.arc_type);
                if !a.description.is_empty() {
                    s.push_str(&format!(" — {}", a.description));
                }
                s
            })
            .collect();
        user.push_str(&arc_strs.join("; "));
        user.push('\n');
    }

    // Parent info.
    user.push_str(&format!(
        "{}: {}\n",
        parent_level.label().to_uppercase(),
        request.parent_node.name,
    ));

    // Parent notes.
    user.push_str(&format!(
        "{} NOTES:\n",
        parent_level.label().to_uppercase()
    ));
    user.push_str(&request.parent_node.content.notes);
    user.push_str("\n\n");

    // Duration context.
    let duration_ms = request.parent_node.time_range.duration_ms();
    let duration_sec = duration_ms / 1000;
    user.push_str(&format!(
        "DURATION: {} seconds ({} minutes {} seconds)\n\n",
        duration_sec,
        duration_sec / 60,
        duration_sec % 60,
    ));

    // Continuity.
    if !request.surrounding_context.preceding_recaps.is_empty() {
        user.push_str("CONTINUITY CONTEXT:\n");
        for entry in &request.surrounding_context.preceding_recaps {
            user.push_str(&format!(
                "- {} / {}: {}\n",
                entry.arc_name, entry.node_name, entry.recap,
            ));
        }
        user.push('\n');
    }

    // JSON format for response.
    let beat_type_field = if child_level == StoryLevel::Beat {
        "\"beat_type\": \"<one of: Setup, Complication, Escalation, Climax, Resolution, Payoff, Callback>\",\n             "
    } else {
        ""
    };

    let entity_fields = if child_level == StoryLevel::Beat || child_level == StoryLevel::Scene {
        ",\n             \"characters\": [\"<character names present>\"],\n             \"location\": \"<scene heading or null if unchanged>\",\n             \"props\": [\"<props characters interact with>\"]"
    } else {
        ""
    };

    user.push_str(&format!(
        "Respond with a JSON array of {}s:\n\
         ```json\n\
         [\n\
           {{\n\
             \"name\": \"<short descriptive name>\",\n\
             {}\
             \"outline\": \"<1-2 sentence description>\",\n\
             \"weight\": <relative duration, e.g. 1.0>{}\n\
           }}\n\
         ]\n\
         ```",
        child_label, beat_type_field, entity_fields,
    ));

    ChatPrompt { system, user }
}
