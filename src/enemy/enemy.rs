use avian2d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

use crate::{
    animation::tank_spawn_animation,
    collision::{add_bullet_collision, add_tank_collision},
    common_component::{
        BulletInfo, EnemyTankSpawn, Facing, MoveAnimation, ROTATION, SpawnAnimation, TankId,
        enemy_loop_animation, spawn_animation,
    },
    config::{
        ENEMY_ACTION_INTERVAL_MAX, ENEMY_ACTION_INTERVAL_MIN, ENEMY_FIRE_CHANCE,
        ENEMY_HEAVY_TANK_MOVE_SPEED, ENEMY_LIGHT_TANK_MOVE_SPEED, ENEMY_TANK_FIRE_INTERVAL,
        ENEMY_TANK_GENERATE_INTERVAL, ENEMY_TANK_TURN_INTERVAL, ENEMY_TURN_CHANCE,
        FAST_TANK_MOVE_SPEED, ICE_MIN_SPEED, MAX_ENEMY_TANK_COUNT_PER_STAGE, TANK_RENDER_Z,
    },
    enemy::{Enemy, EnemyInfo, EnemyNumberState},
    map::{EnemySpawnPos, IceTiles, MapState, apply_ice_movement, world_tile},
    player::{PlayerInfo, PlayerSpawning},
    props::{PropStatus, PropType},
    resource_manage::ImgAsset,
    screens::{Screen, game_is_active},
};

/// 敌人 AI 组件
#[derive(Component)]
pub struct EnemyAI {
    turn_timer: Timer,
    fire_timer: Timer,
    pub tank_type: u8,
}

/// 敌人出生间隔计时器
#[derive(Resource)]
struct EnemySpawnTimer(Timer);

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(EnemySpawnTimer(Timer::from_seconds(
        ENEMY_TANK_GENERATE_INTERVAL,
        TimerMode::Repeating,
    )))
    .add_systems(
        Update,
        (
            spawn_enemy.after(tank_spawn_animation),
            spawn_enemy_animation,
            enemy_move,
        )
            .chain()
            .run_if(
                in_state(MapState::Complete)
                    .and(in_state(Screen::GamePlay))
                    .and(game_is_active),
            ),
    );
}

/// 敌人生成消息动画
fn spawn_enemy_animation(
    mut commands: Commands,
    mut enemy_spawn_query: Query<(Entity, &Transform), With<EnemySpawnPos>>,
    mut enemy_number_state: ResMut<EnemyNumberState>,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    img_asset: Res<ImgAsset>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    tank_transform_query: Query<
        &Transform,
        (
            Without<EnemySpawnPos>,
            Or<(
                With<Enemy>,
                With<SpawnAnimation>,
                (With<PlayerInfo>, Without<PlayerSpawning>),
            )>,
        ),
    >,
) {
    if enemy_number_state.spawned_enemy_count >= 3 {
        spawn_timer.0.tick(time.delta());
        if !spawn_timer.0.just_finished() {
            return;
        }
    }
    // 获取敌人坦克生成点
    for (_, transform) in &mut enemy_spawn_query {
        // 判断场上存在的敌人坦克数量和已生成过的坦克数量
        if enemy_number_state.current_enemy_count < MAX_ENEMY_TANK_COUNT_PER_STAGE
            && enemy_number_state.spawned_enemy_count < enemy_number_state.max_enemy_count
        {
            let spawn_tile = world_tile(transform.translation);
            // 检查出生点所在 16×16 格子是否被占用
            let is_occupied = tank_transform_query
                .iter()
                .any(|t| world_tile(t.translation) == spawn_tile);
            if is_occupied {
                continue;
            }
            let animation_bundle =
                spawn_animation(&img_asset, &mut texture_atlas_layouts, *transform, 0);
            commands.spawn(animation_bundle);
            enemy_number_state.current_enemy_count += 1;
            enemy_number_state.spawned_enemy_count += 1;
            // 每次计时器触发只在一个空闲出生点生成，避免同帧多出生点冲突
            return;
        }
    }
}

/// 创建敌人（将生成动画实体就地转换为敌坦克，不 despawn 后重新 spawn）
fn spawn_enemy(
    mut commands: Commands,
    img_asset: Res<ImgAsset>,
    mut messages: MessageReader<EnemyTankSpawn>,
) {
    for message in messages.read() {
        let Ok(mut entity_commands) = commands.get_entity(message.entity) else {
            continue;
        };

        // 随机坦克类型
        let tank_type = rand::random_range(1..4);
        // 随机是否携带道具
        let is_carrying_prop = rand::random_bool(1. / 3.);
        // 根据类型设置敌人生命值
        let (life, score) = match tank_type {
            3 => (3, 500),
            2 => (1, 200),
            _ => (1, 100),
        };

        let move_animation = enemy_loop_animation(tank_type, is_carrying_prop, &img_asset);
        let first_frame = move_animation.frames[0].clone();
        let spawn_rotation = Quat::from_rotation_z(ROTATION[Facing::Down as usize]);

        entity_commands
            .remove::<(
                SpawnAnimation,
                TankId,
                RigidBody,
                Collider,
                Friction,
                Restitution,
                CollisionEventsEnabled,
                CollisionLayers,
            )>()
            .insert((
                Enemy,
                Sprite {
                    image: first_frame,
                    ..default()
                },
                Transform {
                    translation: Vec3::new(
                        message.transform.translation.x,
                        message.transform.translation.y,
                        TANK_RENDER_Z,
                    ),
                    rotation: spawn_rotation,
                    ..default()
                },
                Rotation::from(spawn_rotation),
                Facing::Down,
                EnemyInfo {
                    level: 0,
                    life,
                    score,
                    carrying_prop: is_carrying_prop,
                },
                EnemyAI {
                    turn_timer: random_initial_timer(ENEMY_TANK_TURN_INTERVAL),
                    fire_timer: random_initial_timer(ENEMY_TANK_FIRE_INTERVAL),
                    tank_type,
                },
                move_animation,
            ))
            .insert(add_tank_collision("enemy"));
    }
}

/// 敌人坦克随机转向移动 + 随机开火
fn enemy_move(
    time: Res<Time>,
    ice_tiles: Option<Res<IceTiles>>,
    mut commands: Commands,
    mut enemy_query: Query<(
        Entity,
        &mut EnemyAI,
        &mut EnemyInfo,
        &mut Facing,
        &mut Transform,
        &mut LinearVelocity,
        &mut MoveAnimation,
        Option<&mut Rotation>,
        Option<&Children>,
    )>,
    img_asset: Res<ImgAsset>,
    mut prop_query: Query<&mut PropStatus>,
) {
    'enemy: for (
        entity,
        mut ai,
        enemy_info,
        mut facing,
        mut transform,
        mut velocity,
        mut animation,
        physics_rotation,
        children,
    ) in enemy_query.iter_mut()
    {
        if let Some(children) = children {
            for child in children.iter() {
                let Ok(mut prop_status) = prop_query.get_mut(child) else {
                    continue;
                };
                if prop_status.prop_type == PropType::Idle {
                    prop_status.timer.tick(time.delta());
                    if prop_status.timer.just_finished() {
                        commands.entity(child).despawn();
                    } else {
                        continue 'enemy;
                    }
                }
            }
        }
        ai.turn_timer.tick(time.delta());
        ai.fire_timer.tick(time.delta());

        // 冷却结束后按概率随机转向
        if ai.turn_timer.just_finished() {
            if rand::random_bool(ENEMY_TURN_CHANCE) {
                let r = rand::random_range(0..4);
                *facing = match r {
                    0 => Facing::Up,
                    1 => Facing::Down,
                    2 => Facing::Left,
                    _ => Facing::Right,
                };
            }
            ai.turn_timer = random_action_timer(ENEMY_TANK_TURN_INTERVAL);
        }

        // 每帧持续施加速度；冰面保留惯性，精灵朝向跟随意图 Facing
        {
            let speed = match ai.tank_type {
                3 => ENEMY_HEAVY_TANK_MOVE_SPEED,
                2 => FAST_TANK_MOVE_SPEED,
                _ => ENEMY_LIGHT_TANK_MOVE_SPEED,
            };

            let direction = match *facing {
                Facing::Up => Vec2::new(0.0, 1.0),
                Facing::Down => Vec2::new(0.0, -1.0),
                Facing::Left => Vec2::new(-1.0, 0.0),
                Facing::Right => Vec2::new(1.0, 0.0),
            };
            let target_velocity = direction * speed;
            let on_ice = ice_tiles
                .as_ref()
                .is_some_and(|tiles| tiles.contains(transform.translation));
            velocity.0 = if on_ice {
                apply_ice_movement(velocity.0, target_velocity, time.delta_secs())
            } else {
                target_velocity
            };
            animation.playing =
                velocity.0.length_squared() > ICE_MIN_SPEED * ICE_MIN_SPEED;

            // 精灵立即跟随意图朝向；冰面上速度可滞后，形成惯性打滑
            let quat = Quat::from_rotation_z(ROTATION[*facing as usize]);
            transform.rotation = quat;
            if let Some(mut physics_rotation) = physics_rotation {
                *physics_rotation = Rotation::from(quat);
            }
        }

        // 冷却结束后按概率随机开火
        if ai.fire_timer.just_finished() {
            if rand::random_bool(ENEMY_FIRE_CHANCE) {
                let fire_pos = match *facing {
                    Facing::Up => {
                        Vec3::new(transform.translation.x, transform.translation.y + 5.0, 1.0)
                    }
                    Facing::Down => {
                        Vec3::new(transform.translation.x, transform.translation.y - 5.0, 1.0)
                    }
                    Facing::Left => {
                        Vec3::new(transform.translation.x - 5.0, transform.translation.y, 1.0)
                    }
                    Facing::Right => {
                        Vec3::new(transform.translation.x + 5.0, transform.translation.y, 1.0)
                    }
                };

                let bullet_vel = match *facing {
                    Facing::Up => Vec2::new(0.0, 100.0),
                    Facing::Down => Vec2::new(0.0, -100.0),
                    Facing::Left => Vec2::new(-100.0, 0.0),
                    Facing::Right => Vec2::new(100.0, 0.0),
                };

                commands.spawn((
                    BulletInfo {
                        entity,
                        level: enemy_info.level,
                        horde: 2, // 敌方阵营
                    },
                    Sprite {
                        image: img_asset.bullet.clone(),
                        ..default()
                    },
                    ActiveCollisionHooks::FILTER_PAIRS,
                    add_bullet_collision(bullet_vel),
                    Transform {
                        translation: fire_pos,
                        rotation: Quat::from_rotation_z(ROTATION[*facing as usize]),
                        ..default()
                    },
                ));
            }
            ai.fire_timer = random_action_timer(ENEMY_TANK_FIRE_INTERVAL);
        }
    }
}

/// 生成带随机时长的单次行动计时器
fn random_action_timer(base_secs: f32) -> Timer {
    Timer::from_seconds(
        base_secs * rand::random_range(ENEMY_ACTION_INTERVAL_MIN..ENEMY_ACTION_INTERVAL_MAX),
        TimerMode::Once,
    )
}

/// 生成带随机初始偏移的计时器，避免多辆坦克同步行动
fn random_initial_timer(base_secs: f32) -> Timer {
    let duration =
        base_secs * rand::random_range(ENEMY_ACTION_INTERVAL_MIN..ENEMY_ACTION_INTERVAL_MAX);
    let mut timer = Timer::from_seconds(duration, TimerMode::Once);
    timer.set_elapsed(Duration::from_secs_f32(rand::random_range(0.0..duration)));
    timer
}
