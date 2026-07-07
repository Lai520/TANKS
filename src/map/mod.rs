use bevy::prelude::*;
pub mod render_map;
use bevy_ecs_ldtk::{
    EntityInstance, LdtkEntity, LdtkIntCell, ldtk::FieldValue, ldtk::LayerInstance,
};

/// 请求重新加载当前 LDtk 关卡（用于进入下一关）
#[derive(Message)]
pub struct ReloadLevel;

pub(super) fn plguin(app: &mut App) {
    app.init_state::<MapState>()
        .insert_resource(MapLevel { value: 0 })
        .add_message::<ReloadLevel>()
        .add_plugins(render_map::plguin);
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum MapState {
    #[default]
    None,
    Drawing,  // 地图绘制中
    Complete, // 地图绘制完毕
}

/// 当前关卡
#[derive(Resource)]
pub struct MapLevel {
    pub value: usize,
}

/// 道具位置计算使用的网格边长
const PROP_GRID_SIZE: f32 = 8.;

/// 当前关卡的尺寸与位置（来自 LDtk）
#[derive(Resource, Clone, Debug)]
pub struct MapBounds {
    pub origin: Vec2,
    /// 8×8 格子内可放置道具的局部坐标（已排除边界层）
    playable_local_positions: Vec<Vec2>,
}

impl MapBounds {
    /// 在可到达区域内随机选取一个 8×8 格子中心点（世界坐标）
    pub fn random_grid_center(&self) -> Vec3 {
        let local = if self.playable_local_positions.is_empty() {
            Vec2::new(PROP_GRID_SIZE, PROP_GRID_SIZE)
        } else {
            let index = rand::random_range(0..self.playable_local_positions.len());
            self.playable_local_positions[index]
        };
        Vec3::new(self.origin.x + local.x, self.origin.y + local.y, 3.)
    }
}

/// 堡垒
#[derive(Component, Default)]
pub struct Camp;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct CampBundle {
    camp: Camp,
}

/// 草地
#[derive(Component, Default)]
pub struct Grass;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct GrassBundle {
    grass: Grass,
}

/// 冰雪地
#[derive(Component, Default)]
pub struct Ice;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct IceBundle {
    ice: Ice,
}

/// 河流
#[derive(Component, Default)]
pub struct River;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct RiverBundle {
    river: River,
}

/// 石头
#[derive(Component, Default)]
pub struct Stone;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct StoneBundle {
    stone: Stone,
}

/// 红砖
#[derive(Component, Default)]
pub struct RedBrick;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct RedBrickBundle {
    red_brick: RedBrick,
}

/// 工兵铲效果期间被隐藏的红砖
#[derive(Component)]
pub struct ShovelHiddenRedBrick;

/// 钢铁
#[derive(Component, Default)]
pub struct Steel;
#[derive(Default, Bundle, LdtkIntCell)]
pub struct SteelBundle {
    steel: Steel,
}

/// 玩家出现位置
#[derive(Component, Default)]
pub struct PlayerSpawnPos(pub u8);
#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[from_entity_instance]
    player_spawn_pos: PlayerSpawnPos,
}

impl From<&EntityInstance> for PlayerSpawnPos {
    fn from(entity_instance: &EntityInstance) -> Self {
        let id = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == "id")
            .and_then(|f| match &f.value {
                FieldValue::Int(Some(value)) => Some(*value),
                _ => None,
            })
            .unwrap_or(1);

        PlayerSpawnPos(id as u8)
    }
}

/// 敌坦克出现位置
#[derive(Component, Default)]
pub struct EnemySpawnPos;
#[derive(Default, Bundle, LdtkEntity)]
pub struct EnemySpawnPosBundle {
    enemy_spawn_pos: EnemySpawnPos,
}

/// 从 Boundary 层 int grid 收集可放置道具的 8×8 格子中心（局部坐标）
pub(super) fn playable_prop_local_positions(boundary: &LayerInstance) -> Vec<Vec2> {
    use bevy_ecs_ldtk::utils::{grid_coords_to_translation, int_grid_index_to_grid_coords};

    let grid_w = boundary.c_wid as u32;
    let grid_h = boundary.c_hei as u32;
    let grid_size = IVec2::splat(PROP_GRID_SIZE as i32);

    boundary
        .int_grid_csv
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            if *value != 0 {
                return None;
            }
            let coords = int_grid_index_to_grid_coords(index, grid_w, grid_h)?;
            Some(grid_coords_to_translation(coords, grid_size))
        })
        .collect()
}
