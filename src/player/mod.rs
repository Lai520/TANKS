use bevy::prelude::*;

use crate::config::PLAYER_IDLE;

mod controller;
mod kill_records;
mod player;

pub use kill_records::{ENEMY_TANK_TYPE_COUNT, EnemyKillRecords};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((player::plugin, controller::plugin))
        .add_systems(Update, player_idle_system);
}

/// 玩家被命中的僵直状态
#[derive(Component)]
pub struct PlayerIdle {
    pub timer: Timer,
    pub flash_timer: Timer,
}

impl Default for PlayerIdle {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(PLAYER_IDLE, TimerMode::Once),
            flash_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

/// 玩家僵直系统：计时结束后恢复，期间闪烁
fn player_idle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerIdle, &mut Sprite)>,
) {
    for (entity, mut idle, mut sprite) in query.iter_mut() {
        idle.timer.tick(time.delta());

        if idle.timer.just_finished() {
            commands.entity(entity).remove::<PlayerIdle>();
            sprite.color.set_alpha(1.0);
        } else {
            idle.flash_timer.tick(time.delta());
            if idle.flash_timer.just_finished() {
                let alpha = if sprite.color.alpha() > 0.5 { 0.3 } else { 1.0 };
                sprite.color.set_alpha(alpha);
            }
        }
    }
}

/// 玩家信息
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerInfo {
    pub id: u8, // 玩家id
    pub level: u8,
    pub life: u8,
    pub chance: u8,
    pub score: usize,
    pub enemy_kills: EnemyKillRecords,
    pub move_sound: Option<Entity>,
}

/// 玩家正在播放出生动画，动画结束前不应再次触发生成
#[derive(Component)]
pub(crate) struct PlayerSpawning;

/// 请求播放玩家出生动画（首次进入或坦克被摧毁后重生）
#[derive(Message)]
pub struct PlayerSpawnRequest {
    pub entity: Entity,
    pub id: u8,
}
