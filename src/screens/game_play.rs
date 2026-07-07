use bevy::prelude::*;

use crate::{
    audio::sound_effect,
    enemy::EnemyNumberState,
    map::Camp,
    player::PlayerInfo,
    resource_manage::{AudioAsset, ImgAsset},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_systems(OnEnter(Screen::GamePlay), play)
        .add_systems(Update, (game_pause, check_stage_complete))
        .add_systems(
            FixedUpdate,
            player_chance.run_if(in_state(Screen::GamePlay)),
        )
        .add_systems(OnEnter(Screen::GameOver), game_over)
        .add_systems(
            FixedUpdate,
            game_over_animation.run_if(in_state(Screen::GameOver)),
        );
}

/// 游戏中状态
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    None,
    Pause,          // 游戏暂停
    GameSettlement, // 关卡结算
}

/// 是否处于正常游戏流程（非结算/暂停）
pub fn game_is_active(state: Res<State<GameState>>) -> bool {
    matches!(*state.get(), GameState::None)
}

/// 开始玩
fn play(
    mut enemy_nember_state: ResMut<EnemyNumberState>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    enemy_nember_state.reset();
    next_game_state.set(GameState::None);
}

/// 暂停/继续
fn game_pause() {}

/// 所有敌人被消灭后进入关卡结算
fn check_stage_complete(
    enemy_number_state: Res<EnemyNumberState>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if !matches!(*game_state.get(), GameState::None) {
        return;
    }
    if enemy_number_state.spawned_enemy_count >= enemy_number_state.max_enemy_count
        && enemy_number_state.current_enemy_count == 0
    {
        next_game_state.set(GameState::GameSettlement);
    }
}

/// 检测玩家机会数
fn player_chance(mut next_state: ResMut<NextState<Screen>>, player_info: Query<&PlayerInfo>) {
    let mut chance = 0;
    for player in player_info {
        chance += player.chance
    }
    if chance == 0 {
        // 两个玩家都没有机会了 游戏结束
        next_state.set(Screen::GameOver);
    }
}

/// 游戏结束组件
#[derive(Component)]
struct GameOver;

/// 游戏结束
fn game_over(
    mut commands: Commands,
    camp_query: Query<&Transform, With<Camp>>,
    img_asset: Res<ImgAsset>,
    audio_asset: Res<AudioAsset>,
) {
    for transform in camp_query {
        let mut game_over_transform = transform.clone();
        game_over_transform.translation.x += 8.;
        game_over_transform.translation.z = 3.;
        game_over_transform.translation.y = -80.;
        commands.spawn((
            GameOver,
            Sprite {
                image: img_asset.gameover.clone(),
                ..default()
            },
            game_over_transform,
        ));
    }
    // 播放gameover音效
    commands.spawn(sound_effect(audio_asset.game_over.clone()));
}

/// 游戏结束动画目标 Y（屏幕中心）
const GAME_OVER_ANIM_TARGET_Y: f32 = 120.0;
/// 游戏结束动画移动速度（像素/秒）
const GAME_OVER_ANIM_SPEED: f32 = 60.0;

/// 游戏结束动画
fn game_over_animation(
    time: Res<Time>,
    mut big_gameover_query: Query<&mut Transform, With<GameOver>>,
) {
    for mut transform in &mut big_gameover_query {
        if transform.translation.y >= GAME_OVER_ANIM_TARGET_Y {
            transform.translation.y = GAME_OVER_ANIM_TARGET_Y;
            continue;
        }

        transform.translation.y = (transform.translation.y
            + GAME_OVER_ANIM_SPEED * time.delta_secs())
        .min(GAME_OVER_ANIM_TARGET_Y);
    }
}
