use bevy::prelude::{
    App, Camera2d, ClearColor, Color, Commands, Component, Plugin, ResMut, Resource, Startup,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeRenderConfig {
    pub borderless_panel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct BibleGraphNativePanelScene {
    pub background_color: &'static str,
    pub grid_color: &'static str,
    pub accent_color: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Resource)]
pub struct BibleGraphNativePanelStatus {
    pub camera_count: usize,
}

impl Default for BibleGraphNativePanelScene {
    fn default() -> Self {
        Self {
            background_color: "#11151d",
            grid_color: "#253041",
            accent_color: "#f2c94c",
        }
    }
}

#[derive(Component)]
pub struct BibleGraphNativeCamera;

impl Default for BibleGraphNativeRenderConfig {
    fn default() -> Self {
        Self {
            borderless_panel: true,
        }
    }
}

pub struct BibleGraphNativeRenderPlugin;

impl Plugin for BibleGraphNativeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BibleGraphNativeRenderConfig::default());
        app.insert_resource(BibleGraphNativePanelScene::default());
        app.insert_resource(BibleGraphNativePanelStatus::default());
        app.insert_resource(ClearColor(Color::srgb(0.067, 0.082, 0.114)));
        app.add_systems(Startup, spawn_bible_graph_native_panel_scene);
    }
}

fn spawn_bible_graph_native_panel_scene(
    mut commands: Commands,
    mut status: ResMut<BibleGraphNativePanelStatus>,
) {
    commands.spawn((Camera2d, BibleGraphNativeCamera));
    status.camera_count = 1;
}
