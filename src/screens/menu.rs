use bevy::prelude::*;
use leafwing_input_manager::{action_state::ActionState, input_map::InputMap};

use crate::{
    audio::sound_effect,
    input_manager::PlayerAction,
    resource_manage::{AudioAsset, ImgAsset},
    screens::{PlayerNumber, Screen},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Menu), spawn_start_menu)
        .add_systems(Update, menu_select.run_if(in_state(Screen::Menu)));
}

/// 菜单
#[derive(Component)]
struct Menu;

/// 菜单光标
#[derive(Component)]
struct Cursor;

/// 开始菜单页
fn spawn_start_menu(mut commands: Commands, img_asset: Res<ImgAsset>) {
    // 菜单控制
    let menu_controller = InputMap::new([
        (PlayerAction::MoveUp, KeyCode::KeyW),
        (PlayerAction::MoveDown, KeyCode::KeyS),
        (PlayerAction::Start, KeyCode::Enter),
    ])
    .with(PlayerAction::MoveUp, GamepadButton::DPadUp)
    .with(PlayerAction::MoveDown, GamepadButton::DPadDown)
    .with(PlayerAction::Start, GamepadButton::Start);

    commands.spawn((
        Menu,
        DespawnOnExit(Screen::Menu),
        Sprite {
            image: img_asset.start_bg.clone(),
            ..default()
        },
        Transform::from_xyz(140., 120., 1.),
        children![(
            Cursor,
            Sprite {
                image: img_asset.p_icon.clone(),
                ..default()
            },
            Transform::from_xyz(-50., -19.5, 2.),
        )],
        menu_controller,
    ));
}

/// 菜单选择
fn menu_select(
    mut commands: Commands,
    mut player_number: ResMut<PlayerNumber>,
    mut next_state: ResMut<NextState<Screen>>,
    audio_asset: Res<AudioAsset>,
    query: Query<&ActionState<PlayerAction>, With<Menu>>,
    mut cursor_query: Query<&mut Transform, With<Cursor>>,
) {
    for mut cursor in cursor_query.iter_mut() {
        for action_state in query.iter() {
            if action_state.just_pressed(&PlayerAction::MoveDown) {
                player_number.value = 2;
                commands.spawn(sound_effect(audio_asset.mode_switch.clone()));
                cursor.translation = Vec3::new(-50., -35.5, 2.);
            } else if action_state.just_pressed(&PlayerAction::MoveUp) {
                player_number.value = 1;
                commands.spawn(sound_effect(audio_asset.mode_switch.clone()));
                cursor.translation = Vec3::new(-50., -19.5, 2.);
            } else if action_state.just_pressed(&PlayerAction::Start) {
                next_state.set(Screen::GamePlay);
                commands.spawn(sound_effect(audio_asset.start_menu.clone()));
            }
        }
    }
}
