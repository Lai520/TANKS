use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapId, map::TilemapTexture, tiles::TileVisible};

use crate::{
    collision::add_wall_collision,
    config::SHOVEL_EFFECT_TIME,
    map::{Camp, RedBrick, ShovelHiddenRedBrick, Stone},
    screens::{game_is_active, Screen},
};

/// 玩家工兵铲生成的临时石墙
#[derive(Component)]
pub struct ShovelStoneMarker;

/// 堡垒周围墙体已被敌人铲毁（永久性）
#[derive(Resource, Default)]
pub struct CampRingWallsCleared(pub bool);

/// 堡垒工兵铲效果状态
#[derive(Resource, Default)]
pub struct CampShovelEffect {
    timer: Timer,
    spawned_stones: Vec<Entity>,
    hidden_red_bricks: Vec<Entity>,
}

impl CampShovelEffect {
    fn is_active(&self) -> bool {
        !self.spawned_stones.is_empty() && !self.timer.is_finished()
    }
}

/// 堡垒周围 8×8 石墙环（相对堡垒中心的世界坐标偏移）
const SHOVEL_RING_OFFSETS: [Vec2; 12] = [
    Vec2::new(-12., -12.),
    Vec2::new(-4., -12.),
    Vec2::new(4., -12.),
    Vec2::new(12., -12.),
    Vec2::new(-12., -4.),
    Vec2::new(12., -4.),
    Vec2::new(-12., 4.),
    Vec2::new(12., 4.),
    Vec2::new(-12., 12.),
    Vec2::new(-4., 12.),
    Vec2::new(4., 12.),
    Vec2::new(12., 12.),
];

const STONE_CELL_HALF: f32 = 4.;
const RED_BRICK_HALF: f32 = 2.;

/// Map 层石墙瓦片在 map.png 中的像素区域
const STONE_TILE_RECT: Rect = Rect {
    min: Vec2::new(32., 0.),
    max: Vec2::new(40., 8.),
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CampShovelEffect>()
        .init_resource::<CampRingWallsCleared>()
        .add_systems(
            OnEnter(Screen::GamePlay),
            (reset_camp_shovel_effect, reset_camp_ring_walls_cleared),
        )
        .add_systems(
            Update,
            tick_camp_shovel_effect.run_if(in_state(Screen::GamePlay).and(game_is_active)),
        );
}

fn reset_camp_ring_walls_cleared(mut cleared: ResMut<CampRingWallsCleared>) {
    cleared.0 = false;
}

fn reset_camp_shovel_effect(mut effect: ResMut<CampShovelEffect>) {
    *effect = CampShovelEffect::default();
}

/// 激活工兵铲效果：Camp 周围 8×8 格一律生成石墙，有红砖则隐藏；连续拾取仅刷新时长
pub fn activate_camp_shovel_effect(
    commands: &mut Commands,
    effect: &mut CampShovelEffect,
    camp_query: &Query<&GlobalTransform, With<Camp>>,
    red_brick_query: &Query<(Entity, &GlobalTransform), With<RedBrick>>,
    stone_tilemap_query: &Query<&TilemapId, With<Stone>>,
    tilemap_texture_query: &Query<&TilemapTexture>,
) {
    if effect.is_active() {
        effect.timer = Timer::from_seconds(SHOVEL_EFFECT_TIME, TimerMode::Once);
        return;
    }

    let Ok(camp_global) = camp_query.single() else {
        return;
    };
    let Some(stone_sprite) = shovel_stone_sprite(stone_tilemap_query, tilemap_texture_query) else {
        return;
    };

    let camp_z = camp_global.translation().z;
    let cell_centers = camp_ring_cell_centers(camp_global);

    let red_bricks: Vec<Entity> = red_brick_query
        .iter()
        .filter(|(_, transform)| {
            let pos = transform.translation().truncate();
            cell_centers
                .iter()
                .any(|center| brick_overlaps_cell(pos, *center))
        })
        .map(|(entity, _)| entity)
        .collect();

    for entity in &red_bricks {
        hide_red_brick(commands, *entity);
    }
    effect.hidden_red_bricks = red_bricks;

    for cell_center in cell_centers {
        let stone = spawn_shovel_stone(commands, &stone_sprite, cell_center, camp_z);
        effect.spawned_stones.push(stone);
    }

    effect.timer = Timer::from_seconds(SHOVEL_EFFECT_TIME, TimerMode::Once);
}

/// 敌人拾取工兵铲：永久销毁堡垒周围红砖与石墙（不销毁边界层钢铁）
pub fn destroy_camp_ring_permanently(
    commands: &mut Commands,
    effect: &mut CampShovelEffect,
    cleared: &mut CampRingWallsCleared,
    camp_query: &Query<&GlobalTransform, With<Camp>>,
    red_brick_query: &Query<(Entity, &GlobalTransform), With<RedBrick>>,
    hidden_red_brick_query: &Query<(Entity, &GlobalTransform), With<ShovelHiddenRedBrick>>,
    stone_query: &Query<(Entity, &GlobalTransform), With<Stone>>,
) {
    let Ok(camp_global) = camp_query.single() else {
        return;
    };
    let cell_centers = camp_ring_cell_centers(camp_global);

    let mut despawned = Vec::new();
    for (entity, transform) in red_brick_query
        .iter()
        .chain(hidden_red_brick_query.iter())
    {
        if in_camp_ring(transform, &cell_centers) {
            despawned.push(entity);
            commands.entity(entity).despawn();
        }
    }
    effect
        .hidden_red_bricks
        .retain(|entity| !despawned.contains(entity));

    for (entity, transform) in stone_query.iter() {
        if in_camp_ring(transform, &cell_centers) {
            despawned.push(entity);
            commands.entity(entity).despawn();
        }
    }
    effect
        .spawned_stones
        .retain(|entity| !despawned.contains(entity));

    if effect.spawned_stones.is_empty() && !effect.timer.is_finished() {
        effect.hidden_red_bricks.clear();
        effect.timer.set_elapsed(effect.timer.duration());
    }

    cleared.0 = true;
}

fn camp_ring_cell_centers(camp_global: &GlobalTransform) -> Vec<Vec2> {
    let camp_center = camp_global.translation().truncate();
    SHOVEL_RING_OFFSETS
        .iter()
        .map(|offset| camp_center + *offset)
        .collect()
}

fn in_camp_ring(transform: &GlobalTransform, cell_centers: &[Vec2]) -> bool {
    let pos = transform.translation().truncate();
    cell_centers
        .iter()
        .any(|center| brick_overlaps_cell(pos, *center))
}

fn shovel_stone_sprite(
    stone_tilemap_query: &Query<&TilemapId, With<Stone>>,
    tilemap_texture_query: &Query<&TilemapTexture>,
) -> Option<Sprite> {
    let tilemap_id = stone_tilemap_query.iter().next()?;
    let TilemapTexture::Single(image) = tilemap_texture_query.get(tilemap_id.0).ok()? else {
        return None;
    };

    Some(Sprite {
        image: image.clone(),
        rect: Some(STONE_TILE_RECT),
        custom_size: Some(Vec2::splat(8.)),
        ..default()
    })
}

fn tick_camp_shovel_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut effect: ResMut<CampShovelEffect>,
    cleared: Res<CampRingWallsCleared>,
) {
    if effect.spawned_stones.is_empty() {
        return;
    }

    effect.timer.tick(time.delta());
    if effect.timer.just_finished() {
        restore_camp_shovel_effect(&mut commands, &mut effect, &cleared);
    }
}

fn restore_camp_shovel_effect(
    commands: &mut Commands,
    effect: &mut CampShovelEffect,
    cleared: &CampRingWallsCleared,
) {
    for stone in effect.spawned_stones.drain(..) {
        if commands.get_entity(stone).is_ok() {
            commands.entity(stone).despawn();
        }
    }

    if cleared.0 {
        effect.hidden_red_bricks.clear();
        return;
    }

    for entity in effect.hidden_red_bricks.drain(..) {
        if commands.get_entity(entity).is_err() {
            continue;
        }
        commands.entity(entity).insert((
            RedBrick,
            TileVisible(true),
            add_wall_collision(4., 4., "red_brick"),
        ));
        commands.entity(entity).remove::<ShovelHiddenRedBrick>();
    }
}

fn spawn_shovel_stone(
    commands: &mut Commands,
    sprite: &Sprite,
    cell_center: Vec2,
    camp_z: f32,
) -> Entity {
    commands
        .spawn((
            ShovelStoneMarker,
            Stone,
            sprite.clone(),
            Transform::from_translation(cell_center.extend(camp_z + 1.)),
            add_wall_collision(8., 8., "stone"),
        ))
        .id()
}

fn hide_red_brick(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert((TileVisible(false), ShovelHiddenRedBrick));
    commands.entity(entity).remove::<(
        RedBrick,
        Collider,
        RigidBody,
        Friction,
        Restitution,
        CollisionLayers,
        ColliderDisabled,
    )>();
}

/// 4×4 红砖是否与 8×8 石墙格重叠
fn brick_overlaps_cell(brick_center: Vec2, cell_center: Vec2) -> bool {
    (brick_center.x - cell_center.x).abs() <= RED_BRICK_HALF + STONE_CELL_HALF
        && (brick_center.y - cell_center.y).abs() <= RED_BRICK_HALF + STONE_CELL_HALF
}
