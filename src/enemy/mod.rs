use bevy::prelude::*;

mod enemy;

pub use enemy::EnemyAI;

use crate::config::MAX_ENEMY_TANK_COUNT;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(EnemyNumberState {
        spawned_enemy_count: 0,
        max_enemy_count: MAX_ENEMY_TANK_COUNT,
        current_enemy_count: 0,
    })
    .add_plugins(enemy::plugin);
}

/// 敌坦克
#[derive(Component)]
pub struct Enemy;

/// 敌坦克信息
#[derive(Component)]
pub struct EnemyInfo {
    pub level: u8,           // 等级--的人也可以摧毁stone墙，但是不能发射两颗炮弹
    pub life: u8,            // 生命
    pub score: usize,        // 敌人分数价值
    pub carrying_prop: bool, // 是否携带道具
}

/// 关卡敌人状态
#[derive(Resource)]
pub struct EnemyNumberState {
    /// 当前关卡已生成的敌人数量
    pub spawned_enemy_count: usize,
    /// 关卡最大敌人数量
    pub max_enemy_count: usize,
    /// 场上存活的敌人数量
    pub current_enemy_count: usize,
}

impl EnemyNumberState {
    /// 重置敌人数量状态
    pub fn reset(&mut self) {
        self.spawned_enemy_count = 0;
        self.current_enemy_count = 0;
        self.max_enemy_count = MAX_ENEMY_TANK_COUNT;
    }
}
