#![deny(unsafe_code)]

mod document;
mod fountain;

use std::{fs, path::PathBuf};

use bevy::prelude::*;
use document::{BlockKind, Document};
use fountain::{parse_fountain, source_name_from_path};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 820;

#[derive(Resource, Debug, Clone)]
struct FixturePaths {
    screenplays_dir: PathBuf,
}

impl Default for FixturePaths {
    fn default() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Self {
            screenplays_dir: manifest_dir.join("../../screenplays/real-movies"),
        }
    }
}

#[derive(Component)]
struct MetricsText;

struct PreviewLine {
    text: String,
    kind: BlockKind,
    indent_px: f32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.095, 0.1, 0.105)))
        .insert_resource(FixturePaths::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Eidetic Bevy Text Editor Experiment".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update_metrics)
        .run();
}

fn setup(mut commands: Commands, fixture_paths: Res<FixturePaths>) {
    commands.spawn(Camera2d);
    let document = load_initial_document(&fixture_paths);
    let preview_lines = preview_lines(&document, 38);

    commands.spawn((
        Text2d::new("Eidetic Bevy Text Editor Experiment"),
        TextFont::from_font_size(30.0),
        TextColor(Color::srgb(0.92, 0.92, 0.88)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(-590.0, 360.0, 0.0),
    ));

    commands.spawn((
        Text2d::new(format!(
            "Source: {}\nBlocks: {}\nFixture directory:\n{}",
            document.source_name,
            document.block_count(),
            fixture_paths.screenplays_dir.display()
        )),
        TextFont::from_font_size(17.0),
        TextColor(Color::srgb(0.68, 0.72, 0.76)),
        bevy::sprite::Anchor::TOP_LEFT,
        TextLayout::new_with_justify(Justify::Left),
        Transform::from_xyz(-590.0, 315.0, 0.0),
    ));

    spawn_document_preview(&mut commands, &preview_lines);

    commands.spawn((
        Text2d::new("metrics pending"),
        TextFont::from_font_size(15.0),
        TextColor(Color::srgb(0.56, 0.88, 0.72)),
        bevy::sprite::Anchor::TOP_LEFT,
        MetricsText,
        Transform::from_xyz(-590.0, -330.0, 0.0),
    ));
}

fn load_initial_document(fixture_paths: &FixturePaths) -> Document {
    let Some(path) = first_fountain_fixture(&fixture_paths.screenplays_dir) else {
        return parse_fountain("Built-in sample", SAMPLE_FOUNTAIN);
    };

    match fs::read_to_string(&path) {
        Ok(source) => parse_fountain(source_name_from_path(&path), &source),
        Err(_) => parse_fountain("Built-in sample", SAMPLE_FOUNTAIN),
    }
}

fn first_fountain_fixture(dir: &PathBuf) -> Option<PathBuf> {
    let mut entries = fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "fountain")
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries.into_iter().next()
}

fn preview_lines(document: &Document, max_lines: usize) -> Vec<PreviewLine> {
    let mut lines = Vec::new();
    for block in &document.blocks {
        if block.kind == BlockKind::TitlePage {
            continue;
        }

        let style = style_for_kind(block.kind);
        for line in wrap_preview_text(&block.plain_text(), style.max_chars) {
            lines.push(PreviewLine {
                text: line,
                kind: block.kind,
                indent_px: style.indent_px,
            });
            if lines.len() >= max_lines {
                return lines;
            }
        }

        if style.blank_after && lines.len() < max_lines {
            lines.push(PreviewLine {
                text: String::new(),
                kind: block.kind,
                indent_px: 0.0,
            });
        }
    }
    lines
}

fn spawn_document_preview(commands: &mut Commands, lines: &[PreviewLine]) {
    let start_y = 230.0;
    let line_height = 18.0;

    for (index, line) in lines.iter().enumerate() {
        commands.spawn((
            Text2d::new(line.text.clone()),
            TextFont::from_font_size(font_size_for_kind(line.kind)),
            color_for_kind(line.kind),
            bevy::sprite::Anchor::TOP_LEFT,
            TextLayout::new_with_justify(Justify::Left),
            Transform::from_xyz(
                -560.0 + line.indent_px,
                start_y - (index as f32 * line_height),
                0.0,
            ),
        ));
    }
}

struct PreviewStyle {
    indent_px: f32,
    max_chars: usize,
    blank_after: bool,
}

fn style_for_kind(kind: BlockKind) -> PreviewStyle {
    match kind {
        BlockKind::SceneHeading => PreviewStyle {
            indent_px: 0.0,
            max_chars: 72,
            blank_after: false,
        },
        BlockKind::Character => PreviewStyle {
            indent_px: 260.0,
            max_chars: 32,
            blank_after: false,
        },
        BlockKind::Dialogue => PreviewStyle {
            indent_px: 170.0,
            max_chars: 44,
            blank_after: true,
        },
        BlockKind::Parenthetical => PreviewStyle {
            indent_px: 210.0,
            max_chars: 34,
            blank_after: false,
        },
        BlockKind::Transition => PreviewStyle {
            indent_px: 380.0,
            max_chars: 30,
            blank_after: true,
        },
        BlockKind::Centered => PreviewStyle {
            indent_px: 210.0,
            max_chars: 42,
            blank_after: true,
        },
        BlockKind::TitlePage | BlockKind::Action => PreviewStyle {
            indent_px: 0.0,
            max_chars: 70,
            blank_after: true,
        },
    }
}

fn font_size_for_kind(kind: BlockKind) -> f32 {
    match kind {
        BlockKind::SceneHeading | BlockKind::Character | BlockKind::Transition => 16.5,
        _ => 16.0,
    }
}

fn color_for_kind(kind: BlockKind) -> TextColor {
    let color = match kind {
        BlockKind::SceneHeading => Color::srgb(0.95, 0.93, 0.78),
        BlockKind::Character => Color::srgb(0.78, 0.9, 0.95),
        BlockKind::Dialogue | BlockKind::Parenthetical => Color::srgb(0.9, 0.9, 0.86),
        BlockKind::Transition => Color::srgb(0.92, 0.74, 0.62),
        _ => Color::srgb(0.82, 0.84, 0.82),
    };
    TextColor(color)
}

fn wrap_preview_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let separator = usize::from(!current.is_empty());
        if !current.is_empty()
            && current.chars().count() + separator + word.chars().count() > max_chars
        {
            lines.push(current);
            current = String::new();
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn update_metrics(time: Res<Time>, metrics: Single<&mut Text2d, With<MetricsText>>) {
    let delta_ms = time.delta_secs_f64() * 1000.0;
    let fps = if delta_ms > 0.0 {
        1000.0 / delta_ms
    } else {
        0.0
    };
    let mut metrics = metrics.into_inner();
    metrics.0 = format!("frame: {delta_ms:.2} ms | approx fps: {fps:.1}");
}

const SAMPLE_FOUNTAIN: &str = r#"Title: Built-in Sample

FADE IN:

INT. TEST ROOM - DAY
The editor experiment starts with a fallback script when local fixtures are absent.

MARA
(testing)
The window works. Now make the text useful.

> CUT TO:
"#;
