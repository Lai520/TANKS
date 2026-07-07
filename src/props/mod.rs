use bevy::prelude::*;

use crate::{common_component::PropsSpawn, props::prop_generate::Prop};

mod enemy_pickup;
mod player_pickup;
mod prop_generate;
mod shovel_effect;

pub(super) fn plugin(app: &mut App) {
    app.add_message::<PropsSpawn>().add_plugins((
        prop_generate::plugin,
        shovel_effect::plugin,
        player_pickup::plugin,
        enemy_pickup::plugin,
    ));
}

/// 道具类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropType {
    Chance,      // 添加机会
    Idle,        // 定时待机
    Shovel,      // 工兵铲
    Boom,        // 炸弹
    Star,        // 星星升级
    Bulletproof, // 防弹
    Gun,         // 手枪
    Waterproof,  // 防水层
}

impl PropType {
    pub(super) fn random() -> Self {
        let prop_type = rand::random_range(0..8);
        match prop_type {
            0 => PropType::Chance,
            1 => PropType::Idle,
            2 => PropType::Shovel,
            3 => PropType::Boom,
            4 => PropType::Star,
            5 => PropType::Bulletproof,
            6 => PropType::Gun,
            7 => PropType::Waterproof,
            _ => PropType::Boom,
        }
    }
}

/// 道具效果
#[derive(Component)]
pub struct PropStatus {
    pub timer: Timer,
    pub prop_type: PropType,
}

/// 区分道具实体——道具碰撞组只和玩家、敌人碰撞
fn distinguish_prop_entities(
    e1: Entity,
    e2: Entity,
    prop_query: &Query<&Prop>,
) -> (Entity, Entity, bool) {
    match (prop_query.get(e1), prop_query.get(e2)) {
        (Ok(_), Err(_)) => (e1, e2, true),
        (Err(_), Ok(_)) => (e2, e1, true),
        _ => (e1, e2, false),
    }
}
