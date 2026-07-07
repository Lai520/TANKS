use bevy::{camera::ScalingMode, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
}

/// 添加相机
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(140., 120., 1.),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 240 as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
