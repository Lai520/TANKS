use bevy::{prelude::*, window::WindowResolution};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{ResourceInspectorPlugin, WorldInspectorPlugin},
};

pub const BASE_WIDTH: u32 = 900;
pub const BASE_HEIGHT: u32 = 720;

/// 窗口配置
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(BASE_WIDTH, BASE_HEIGHT),
                    resizable: true,
                    title: "坦克大战".into(),
                    ..default()
                }),
                ..default()
            }),
    );
    // .add_plugins(EguiPlugin::default())
    // .add_plugins(WorldInspectorPlugin::new())
    // .add_plugins(ResourceInspectorPlugin::<Time>::default());
}
