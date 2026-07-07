use bevy::prelude::*;
use bevy_ecs_ldtk::{
    LdtkWorldBundle, LevelSelection,
    app::{LdtkEntityAppExt, LdtkIntCellAppExt},
    assets::{LdtkProject, LevelIndices},
    ldtk::raw_level_accessor::RawLevelAccessor,
    prelude::{LdtkProjectHandle, LevelIid},
};

use crate::{
    collision::add_wall_collision,
    map::{
        Camp, CampBundle, EnemySpawnPosBundle, Grass, GrassBundle, Ice, IceBundle, MapBounds,
        PlayerBundle, RedBrick, RedBrickBundle, River, RiverBundle, Steel, SteelBundle,
        Stone, StoneBundle, playable_prop_local_positions,
    },
    map::{MapLevel, MapState, ReloadLevel},
    resource_manage::ImgAsset,
    screens::Screen,
};

pub(super) fn plguin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::GamePlay),
        (render_map, set_level, clear_map_bounds),
    )
        .register_ldtk_int_cell::<GrassBundle>(1)
        .register_ldtk_int_cell::<IceBundle>(2)
        .register_ldtk_int_cell::<RiverBundle>(3)
        .register_ldtk_int_cell::<StoneBundle>(4)
        .register_ldtk_int_cell_for_layer::<RedBrickBundle>("RedWall", 1)
        .register_ldtk_int_cell_for_layer::<SteelBundle>("Boundary", 1)
        .register_ldtk_int_cell_for_layer::<CampBundle>("Camp", 1)
        .register_ldtk_entity::<PlayerBundle>("Player") // 玩家和敌人实体层
        .register_ldtk_entity::<EnemySpawnPosBundle>("Enemy")
        .add_systems(OnEnter(MapState::Complete), add_cell_collision)
        .add_systems(
            Update,
            (
                init_map_bounds
                    .run_if(in_state(Screen::GamePlay).and(in_state(MapState::Complete)))
                    .run_if(not(resource_exists::<MapBounds>)),
                reload_level.run_if(in_state(Screen::GamePlay)),
            ),
        );
}

fn reload_level(
    mut commands: Commands,
    mut messages: MessageReader<ReloadLevel>,
    img_asset: Res<ImgAsset>,
    map_level: Res<MapLevel>,
    ldtk_world_query: Query<Entity, With<LdtkProjectHandle>>,
    mut map_state: ResMut<NextState<MapState>>,
) {
    for _ in messages.read() {
        for entity in ldtk_world_query.iter() {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<MapBounds>();
        map_state.set(MapState::Drawing);
        commands.spawn(LdtkWorldBundle {
            ldtk_handle: img_asset.map.clone().into(),
            ..default()
        });
        commands.insert_resource(LevelSelection::index(map_level.value));
        map_state.set(MapState::Complete);
    }
}

/// 渲染地图
fn render_map(
    mut commands: Commands,
    img_asset: Res<ImgAsset>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    next_state.set(MapState::Drawing);
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: img_asset.map.clone().into(),
        ..default()
    });
    next_state.set(MapState::Complete);
}

/// 设置关卡
fn set_level(mut commands: Commands, map_level: Res<MapLevel>) {
    commands.insert_resource(LevelSelection::index(map_level.value));
}

fn clear_map_bounds(mut commands: Commands) {
    commands.remove_resource::<MapBounds>();
}

/// 从 LDtk 读取当前关卡的尺寸与位置
fn init_map_bounds(
    mut commands: Commands,
    map_level: Res<MapLevel>,
    ldtk_projects: Res<Assets<LdtkProject>>,
    img_asset: Res<ImgAsset>,
    level_query: Query<&GlobalTransform, With<LevelIid>>,
) {
    let Some(project) = ldtk_projects.get(&img_asset.map) else {
        return;
    };
    let indices = LevelIndices::in_root(map_level.value);
    let Some(level) = project.get_raw_level_at_indices(&indices) else {
        return;
    };
    let Some(level_transform) = level_query.iter().next() else {
        return;
    };

    let playable_local_positions = level
        .layer_instances
        .as_ref()
        .and_then(|layers| layers.iter().find(|layer| layer.identifier == "Boundary"))
        .map(playable_prop_local_positions)
        .unwrap_or_default();

    commands.insert_resource(MapBounds {
        origin: level_transform.translation().truncate(),
        width: level.px_wid as f32,
        height: level.px_hei as f32,
        playable_local_positions,
    });
}

/// 添加墙体碰撞信息
fn add_cell_collision(
    mut commands: Commands,
    stone: Query<Entity, With<Stone>>,
    red_brick: Query<Entity, With<RedBrick>>,
    river: Query<Entity, With<River>>,
    steel: Query<Entity, With<Steel>>,
    camp: Query<Entity, With<Camp>>,
) {
    // 堡垒
    for entity in camp.iter() {
        commands
            .entity(entity)
            .insert(add_wall_collision(16., 16., "camp"));
    }
    // 边界碰撞
    for entity in steel.iter() {
        commands
            .entity(entity)
            .insert(add_wall_collision(8., 8., "steel"));
    }
    // 石头墙碰撞
    for entity in stone.iter() {
        commands
            .entity(entity)
            .insert(add_wall_collision(8., 8., "stone"));
    }
    // 红砖墙碰撞
    for entity in red_brick.iter() {
        commands
            .entity(entity)
            .insert(add_wall_collision(4., 4., "red_brick"));
    }
    // 河流碰撞---子弹可以穿过
    for entity in river.iter() {
        commands
            .entity(entity)
            .insert(add_wall_collision(8., 8., "river"));
    }
}
