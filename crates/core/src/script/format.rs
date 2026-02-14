use crate::script::element::ScriptElement;

/// Formatting rules for 30-minute TV screenplay format.
pub struct FormatRules {
    /// Lines per page (standard: ~56).
    pub lines_per_page: usize,
    /// Max characters per action line (~60).
    pub chars_per_line_action: usize,
    /// Max characters per dialogue line (~35).
    pub chars_per_line_dialogue: usize,
}

impl Default for FormatRules {
    fn default() -> Self {
        Self {
            lines_per_page: 56,
            chars_per_line_action: 60,
            chars_per_line_dialogue: 35,
        }
    }
}

/// Parser state for the script element state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    /// Expecting any element type.
    Start,
    /// Just saw a Character cue; expecting Parenthetical or Dialogue.
    AfterCharacter,
    /// Inside dialogue lines (until blank line or new element).
    InDialogue,
}

/// Parse raw screenplay text into structured `ScriptElement`s.
///
/// Recognizes standard formatting conventions:
/// - Lines starting with `INT.` or `EXT.` → `SceneHeading`
/// - ALL CAPS short lines (≤40 chars, mostly uppercase letters) → `Character`
/// - Lines in parentheses after a Character → `Parenthetical`
/// - Lines after Character/Parenthetical → `Dialogue` (until blank line)
/// - Lines ending with `TO:` → `Transition`
/// - Everything else → `Action`
pub fn parse_script_elements(raw: &str) -> Vec<ScriptElement> {
    let mut elements = Vec::new();
    let mut state = ParseState::Start;
    let mut dialogue_buf = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();

        // Blank line: flush dialogue if active, reset state.
        if trimmed.is_empty() {
            if !dialogue_buf.is_empty() {
                elements.push(ScriptElement::Dialogue(dialogue_buf.trim().to_owned()));
                dialogue_buf.clear();
            }
            state = ParseState::Start;
            continue;
        }

        match state {
            ParseState::Start => {
                if is_scene_heading(trimmed) {
                    elements.push(ScriptElement::SceneHeading(trimmed.to_owned()));
                } else if is_transition(trimmed) {
                    elements.push(ScriptElement::Transition(trimmed.to_owned()));
                } else if is_character_cue(trimmed) {
                    elements.push(ScriptElement::Character(trimmed.to_owned()));
                    state = ParseState::AfterCharacter;
                } else {
                    elements.push(ScriptElement::Action(trimmed.to_owned()));
                }
            }
            ParseState::AfterCharacter => {
                if is_parenthetical(trimmed) {
                    let inner = trimmed
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .to_owned();
                    elements.push(ScriptElement::Parenthetical(inner));
                    // Still expecting dialogue after parenthetical.
                } else {
                    // First line of dialogue.
                    dialogue_buf.push_str(trimmed);
                    state = ParseState::InDialogue;
                }
            }
            ParseState::InDialogue => {
                if is_parenthetical(trimmed) {
                    // Flush current dialogue, emit parenthetical, continue.
                    if !dialogue_buf.is_empty() {
                        elements.push(ScriptElement::Dialogue(dialogue_buf.trim().to_owned()));
                        dialogue_buf.clear();
                    }
                    let inner = trimmed
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .to_owned();
                    elements.push(ScriptElement::Parenthetical(inner));
                } else if is_character_cue(trimmed) {
                    // New character cue inside dialogue block.
                    if !dialogue_buf.is_empty() {
                        elements.push(ScriptElement::Dialogue(dialogue_buf.trim().to_owned()));
                        dialogue_buf.clear();
                    }
                    elements.push(ScriptElement::Character(trimmed.to_owned()));
                    state = ParseState::AfterCharacter;
                } else {
                    // Continuation of dialogue.
                    if !dialogue_buf.is_empty() {
                        dialogue_buf.push(' ');
                    }
                    dialogue_buf.push_str(trimmed);
                }
            }
        }
    }

    // Flush any remaining dialogue.
    if !dialogue_buf.is_empty() {
        elements.push(ScriptElement::Dialogue(dialogue_buf.trim().to_owned()));
    }

    elements
}

/// Estimate page count from a list of script elements.
///
/// Uses standard TV screenplay page estimation:
/// - Each element contributes a line count based on word-wrapped length.
/// - A page is approximately 56 lines.
pub fn estimate_page_count(elements: &[ScriptElement], rules: &FormatRules) -> f64 {
    let mut total_lines: usize = 0;

    for element in elements {
        total_lines += match element {
            ScriptElement::SceneHeading(_) => 2, // heading + blank line after
            ScriptElement::Action(s) => {
                wrapped_line_count(s, rules.chars_per_line_action) + 1
            }
            ScriptElement::Character(_) => 1,
            ScriptElement::Parenthetical(_) => 1,
            ScriptElement::Dialogue(s) => {
                wrapped_line_count(s, rules.chars_per_line_dialogue)
            }
            ScriptElement::Transition(_) => 2, // transition + blank line after
        };
    }

    total_lines as f64 / rules.lines_per_page as f64
}

/// Count how many lines text occupies when wrapped at `max_chars`.
fn wrapped_line_count(text: &str, max_chars: usize) -> usize {
    if text.is_empty() {
        return 1;
    }
    let chars = text.len();
    (chars + max_chars - 1) / max_chars
}

/// Check if a line is a scene heading (starts with INT. or EXT.).
fn is_scene_heading(line: &str) -> bool {
    let upper = line.to_uppercase();
    upper.starts_with("INT.") || upper.starts_with("EXT.") || upper.starts_with("INT/EXT.")
}

/// Check if a line is a transition (ends with TO:).
fn is_transition(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    upper.ends_with("TO:") && upper.len() <= 30
}

/// Check if a line is a character cue.
///
/// Character cues are short ALL CAPS lines, optionally with extensions
/// like `(V.O.)`, `(O.S.)`, `(CONT'D)`.
fn is_character_cue(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() > 40 {
        return false;
    }

    // Strip extension in parentheses for the check.
    let name_part = if let Some(paren_start) = trimmed.find('(') {
        trimmed[..paren_start].trim()
    } else {
        trimmed
    };

    if name_part.is_empty() {
        return false;
    }

    // Must be mostly uppercase letters (allow spaces, hyphens, apostrophes, periods).
    let alpha_chars: Vec<char> = name_part.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha_chars.is_empty() {
        return false;
    }

    let uppercase_count = alpha_chars.iter().filter(|c| c.is_uppercase()).count();
    uppercase_count as f64 / alpha_chars.len() as f64 > 0.8
}

/// Check if a line is a parenthetical direction.
fn is_parenthetical(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('(') && trimmed.ends_with(')')
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SCRIPT: &str = "\
INT. JERRY'S APARTMENT - DAY

Jerry is pacing. George enters.

JERRY
I can't believe you did that.

GEORGE
(defensive)
What? It was a perfectly reasonable
thing to do.

JERRY
You ate the last muffin, George. The
last one.

CUT TO:

EXT. COFFEE SHOP - DAY

Elaine is sitting outside.";

    #[test]
    fn parse_sample_screenplay() {
        let elements = parse_script_elements(SAMPLE_SCRIPT);

        assert!(matches!(&elements[0], ScriptElement::SceneHeading(s) if s.contains("JERRY")));
        assert!(matches!(&elements[1], ScriptElement::Action(s) if s.contains("Jerry is pacing")));
        assert!(matches!(&elements[2], ScriptElement::Character(s) if s == "JERRY"));
        assert!(matches!(&elements[3], ScriptElement::Dialogue(s) if s.contains("can't believe")));
        assert!(matches!(&elements[4], ScriptElement::Character(s) if s == "GEORGE"));
        assert!(matches!(&elements[5], ScriptElement::Parenthetical(s) if s == "defensive"));
        assert!(matches!(&elements[6], ScriptElement::Dialogue(s) if s.contains("perfectly reasonable")));
        assert!(matches!(&elements[7], ScriptElement::Character(s) if s == "JERRY"));
        assert!(matches!(&elements[8], ScriptElement::Dialogue(s) if s.contains("last muffin")));
        assert!(matches!(&elements[9], ScriptElement::Transition(s) if s.contains("CUT TO")));
        assert!(matches!(&elements[10], ScriptElement::SceneHeading(s) if s.contains("COFFEE SHOP")));
        assert!(matches!(&elements[11], ScriptElement::Action(s) if s.contains("Elaine")));
    }

    #[test]
    fn parse_character_with_extension() {
        let input = "JAKE (V.O.)\nSomething isn't right.\n";
        let elements = parse_script_elements(input);

        assert!(matches!(&elements[0], ScriptElement::Character(s) if s == "JAKE (V.O.)"));
        assert!(matches!(&elements[1], ScriptElement::Dialogue(s) if s.contains("isn't right")));
    }

    #[test]
    fn estimate_page_count_roughly_correct() {
        let elements = parse_script_elements(SAMPLE_SCRIPT);
        let pages = estimate_page_count(&elements, &FormatRules::default());
        // A short sample — should be well under 1 page.
        assert!(pages > 0.0 && pages < 1.0, "got {pages}");
    }

    #[test]
    fn scene_heading_detection() {
        assert!(is_scene_heading("INT. LIVING ROOM - DAY"));
        assert!(is_scene_heading("EXT. PARK - NIGHT"));
        assert!(is_scene_heading("INT/EXT. CAR - DAY"));
        assert!(!is_scene_heading("INTERIOR DESIGN"));
        assert!(!is_scene_heading("Hello world"));
    }

    #[test]
    fn character_cue_detection() {
        assert!(is_character_cue("JERRY"));
        assert!(is_character_cue("JAKE (V.O.)"));
        assert!(is_character_cue("MR. SMITH (CONT'D)"));
        assert!(!is_character_cue("This is a regular sentence that happens to be somewhat long."));
        assert!(!is_character_cue("hello"));
    }

    #[test]
    fn transition_detection() {
        assert!(is_transition("CUT TO:"));
        assert!(is_transition("SMASH CUT TO:"));
        assert!(is_transition("FADE TO:"));
        assert!(!is_transition("Going to the store to buy things and more things TO:"));
    }
}
