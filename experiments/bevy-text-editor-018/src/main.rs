#![deny(unsafe_code)]

use std::path::PathBuf;

use bevy::prelude::*;

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

    commands.spawn((
        Text2d::new("Eidetic Bevy Text Editor Experiment"),
        TextFont::from_font_size(30.0),
        TextColor(Color::srgb(0.92, 0.92, 0.88)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(-590.0, 360.0, 0.0),
    ));

    commands.spawn((
        Text2d::new(format!(
            "Standalone Bevy 0.18 native window\nFixture directory:\n{}",
            fixture_paths.screenplays_dir.display()
        )),
        TextFont::from_font_size(17.0),
        TextColor(Color::srgb(0.68, 0.72, 0.76)),
        bevy::sprite::Anchor::TOP_LEFT,
        TextLayout::new_with_justify(Justify::Left),
        Transform::from_xyz(-590.0, 315.0, 0.0),
    ));

    commands.spawn((
        Text2d::new("metrics pending"),
        TextFont::from_font_size(15.0),
        TextColor(Color::srgb(0.56, 0.88, 0.72)),
        bevy::sprite::Anchor::TOP_LEFT,
        MetricsText,
        Transform::from_xyz(-590.0, -330.0, 0.0),
    ));
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
