use eidetic_core::script::element::ScriptElement;
use eidetic_core::script::format::parse_script_elements;
use eidetic_core::timeline::node::StoryLevel;
use eidetic_core::Project;
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
        if let Ok(family) = genpdf::fonts::from_files(
            dir,
            "LiberationMono",
            Some(genpdf::fonts::Builtin::Courier),
        ) {
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

/// Generate a formatted screenplay PDF from a project.
///
/// Follows standard TV screenplay conventions:
/// - Courier 12pt on US Letter (8.5" x 11")
/// - 1.5" left margin, 1" right margin
/// - Scene headings bold ALL CAPS
/// - Character names centered ALL CAPS
/// - Dialogue indented (center-aligned approximation)
/// - Transitions right-aligned
pub fn generate_screenplay_pdf(project: &Project) -> Result<Vec<u8>, String> {
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
    let title = Paragraph::new(&project.name).aligned(Alignment::Center);
    doc.push(StyledElement::new(title, Style::new().bold()));
    doc.push(Break::new(1.0));

    // Start new page for content.
    doc.push(PageBreak::new());

    // Gather Beat-level nodes sorted by time — these contain the scripts.
    let mut beats: Vec<_> = project
        .timeline
        .nodes_at_level(StoryLevel::Beat)
        .into_iter()
        .cloned()
        .collect();
    beats.sort_by_key(|n| n.time_range.start_ms);

    for beat in &beats {
        if beat.content.content.is_empty() {
            continue;
        }
        let text = &beat.content.content;

        let elements = parse_script_elements(text);
        for elem in &elements {
            render_element(&mut doc, elem);
        }
    }

    let mut buf = Vec::new();
    doc.render(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
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
