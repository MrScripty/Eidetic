use eidetic_core::contracts::{ScriptBlockKind, ScriptDocumentProjection, ScriptSegmentProjection};
use eidetic_core::script::element::ScriptElement;
use eidetic_core::script::format::parse_script_elements;
use genpdf::elements::{Break, PageBreak, Paragraph, StyledElement};
use genpdf::fonts::FontFamily;
use genpdf::style::Style;
use genpdf::{Alignment, Document, Margins, SimplePageDecorator, Size};

/// Common font search paths on Linux.
const FONT_SEARCH_DIRS: &[&str] = &[
    "/usr/share/fonts/truetype/liberation",
    "/usr/share/fonts/liberation-mono",
    "/usr/share/fonts/truetype/liberation2",
    "/usr/share/fonts/TTF",
];

/// Try to load Liberation Mono (metrically identical to Courier) from system fonts.
fn load_font_family() -> Result<FontFamily<genpdf::fonts::FontData>, String> {
    for dir in FONT_SEARCH_DIRS {
        if let Ok(family) =
            genpdf::fonts::from_files(dir, "LiberationMono", Some(genpdf::fonts::Builtin::Courier))
        {
            return Ok(family);
        }
    }
    // Fallback: try without builtin flag (will embed the font instead).
    for dir in FONT_SEARCH_DIRS {
        if let Ok(family) = genpdf::fonts::from_files(dir, "LiberationMono", None) {
            return Ok(family);
        }
    }
    Err(
        "Could not find LiberationMono fonts. Install fonts-liberation or liberation-mono-fonts."
            .into(),
    )
}

/// Generate a formatted screenplay PDF from a backend-owned script document projection.
///
/// Follows standard TV screenplay conventions:
/// - Courier 12pt on US Letter (8.5" x 11")
/// - 1.5" left margin, 1" right margin
/// - Scene headings bold ALL CAPS
/// - Character names centered ALL CAPS
/// - Dialogue indented (center-aligned approximation)
/// - Transitions right-aligned
pub fn generate_screenplay_pdf(
    project_name: &str,
    projection: &ScriptDocumentProjection,
) -> Result<Vec<u8>, String> {
    let font_family = load_font_family()?;

    let mut doc = Document::new(font_family);
    doc.set_font_size(12);

    // US Letter in mm: 215.9 x 279.4.
    doc.set_paper_size(Size::new(215.9, 279.4));

    // Margins via page decorator.
    // Left 1.5" = 38.1mm, Right 1" = 25.4mm, Top/Bottom 1" = 25.4mm.
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(25.4, 25.4, 25.4, 38.1));
    doc.set_page_decorator(decorator);

    // Title page.
    doc.push(Break::new(8.0));
    let title = Paragraph::new(project_name).aligned(Alignment::Center);
    doc.push(StyledElement::new(title, Style::new().bold()));
    doc.push(Break::new(1.0));

    // Start new page for content.
    doc.push(PageBreak::new());

    for elem in script_document_elements(projection) {
        render_element(&mut doc, &elem);
    }

    let mut buf = Vec::new();
    doc.render(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

pub(crate) fn script_document_elements(
    projection: &ScriptDocumentProjection,
) -> Vec<ScriptElement> {
    projection
        .segments
        .iter()
        .flat_map(segment_elements)
        .collect()
}

fn segment_elements(segment: &ScriptSegmentProjection) -> Vec<ScriptElement> {
    segment
        .blocks
        .iter()
        .flat_map(|block| {
            let text = block.block.text.trim();
            if text.is_empty() {
                return Vec::new();
            }
            match block.block.block_kind {
                ScriptBlockKind::SceneHeading => {
                    vec![ScriptElement::SceneHeading(text.to_string())]
                }
                ScriptBlockKind::Action => parse_script_elements(text),
                ScriptBlockKind::Character => vec![ScriptElement::Character(text.to_string())],
                ScriptBlockKind::Parenthetical => {
                    vec![ScriptElement::Parenthetical(
                        text.trim_start_matches('(')
                            .trim_end_matches(')')
                            .to_string(),
                    )]
                }
                ScriptBlockKind::Dialogue => vec![ScriptElement::Dialogue(text.to_string())],
                ScriptBlockKind::Transition => vec![ScriptElement::Transition(text.to_string())],
                ScriptBlockKind::Shot | ScriptBlockKind::Note => {
                    vec![ScriptElement::Action(text.to_string())]
                }
            }
        })
        .collect()
}

fn render_element(doc: &mut Document, elem: &ScriptElement) {
    match elem {
        ScriptElement::SceneHeading(s) => {
            doc.push(Break::new(0.5));
            let p = Paragraph::new(s.to_uppercase());
            doc.push(StyledElement::new(p, Style::new().bold()));
            doc.push(Break::new(0.5));
        }
        ScriptElement::Action(s) => {
            doc.push(Paragraph::new(s.as_str()));
            doc.push(Break::new(0.3));
        }
        ScriptElement::Character(s) => {
            doc.push(Break::new(0.3));
            let p = Paragraph::new(s.to_uppercase()).aligned(Alignment::Center);
            doc.push(p);
        }
        ScriptElement::Parenthetical(s) => {
            let text = format!("({s})");
            let p = Paragraph::new(text).aligned(Alignment::Center);
            doc.push(p);
        }
        ScriptElement::Dialogue(s) => {
            let p = Paragraph::new(s.as_str()).aligned(Alignment::Center);
            doc.push(p);
        }
        ScriptElement::Transition(s) => {
            doc.push(Break::new(0.3));
            let p = Paragraph::new(s.to_uppercase()).aligned(Alignment::Right);
            doc.push(p);
            doc.push(Break::new(0.3));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        ScriptBlock, ScriptBlockId, ScriptBlockProjection, ScriptDocument, ScriptDocumentId,
        ScriptSegment, ScriptSegmentId, ScriptSegmentStatus,
    };

    #[test]
    fn script_document_elements_follow_projection_order() {
        let projection = script_projection(vec![
            (ScriptBlockKind::SceneHeading, "INT. KITCHEN - MORNING"),
            (ScriptBlockKind::Action, "Ada enters with a wet umbrella."),
            (ScriptBlockKind::Character, "ADA"),
            (ScriptBlockKind::Parenthetical, "(quietly)"),
            (ScriptBlockKind::Dialogue, "It followed me home."),
            (ScriptBlockKind::Transition, "CUT TO:"),
        ]);

        let elements = script_document_elements(&projection);

        assert_eq!(
            elements,
            vec![
                ScriptElement::SceneHeading("INT. KITCHEN - MORNING".to_string()),
                ScriptElement::Action("Ada enters with a wet umbrella.".to_string()),
                ScriptElement::Character("ADA".to_string()),
                ScriptElement::Parenthetical("quietly".to_string()),
                ScriptElement::Dialogue("It followed me home.".to_string()),
                ScriptElement::Transition("CUT TO:".to_string()),
            ]
        );
    }

    #[test]
    fn script_document_elements_drop_empty_blocks_and_parse_action_text() {
        let projection = script_projection(vec![
            (ScriptBlockKind::Action, "   "),
            (ScriptBlockKind::Action, "EXT. BEACH - DAY\n\nAda runs."),
        ]);

        let elements = script_document_elements(&projection);

        assert_eq!(
            elements,
            vec![
                ScriptElement::SceneHeading("EXT. BEACH - DAY".to_string()),
                ScriptElement::Action("Ada runs.".to_string()),
            ]
        );
    }

    fn script_projection(blocks: Vec<(ScriptBlockKind, &str)>) -> ScriptDocumentProjection {
        ScriptDocumentProjection {
            document: ScriptDocument {
                id: ScriptDocumentId::new("script.document.main").unwrap(),
                title: "Pilot".to_string(),
                sort_order: 0,
            },
            segments: vec![ScriptSegmentProjection {
                segment: ScriptSegment {
                    id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                    document_id: ScriptDocumentId::new("script.document.main").unwrap(),
                    source_node_id: Some("node.beat.opening".to_string()),
                    start_ms: 1_000,
                    end_ms: 5_000,
                    status: ScriptSegmentStatus::Current,
                    sort_order: 1,
                },
                blocks: blocks
                    .into_iter()
                    .enumerate()
                    .map(|(index, (block_kind, text))| ScriptBlockProjection {
                        block: ScriptBlock {
                            id: ScriptBlockId::new(format!("script.block.{index}")).unwrap(),
                            segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                            block_kind,
                            text: text.to_string(),
                            sort_order: index as u32,
                        },
                        spans: Vec::new(),
                        locks: Vec::new(),
                    })
                    .collect(),
            }],
        }
    }
}
