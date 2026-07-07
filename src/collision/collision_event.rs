use avian2d::{collision::collision_events::CollisionStart, prelude::Collider};
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapTexture;
use std::collections::HashSet;

use crate::{
    audio::{self, sound_effect},
    collision::{BoomMsg, ShouldDespawn, strip_tank_physics},
    common_component::{
        BulletInfo, BulletproofLayer, EffectAnimation, Facing, MoveAnimation, PropsSpawn,
        SpawnAnimation, Waterproofer,
    },
    enemy::{EnemyAI, EnemyInfo, EnemyNumberState},
    map::{Camp, RedBrick, ShovelHiddenRedBrick, Steel, Stone},
    player::{PlayerIdle, PlayerInfo, PlayerSpawnRequest},
    resource_manage::{AudioAsset, ImgAsset},
    screens::{game_is_active, Screen},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CampDestroyPending>()
        .init_resource::<BulletsConsumedThisFrame>()
        .add_message::<BoomMsg>()
        .add_systems(OnEnter(Screen::GamePlay), reset_camp_destroy_pending)
        .add_systems(
            Update,
            (
                (
                    bullet_vs_bullet_system,
                    bullet_vs_camp_system,
                    bullet_vs_stone_system,
                    bullet_vs_steel_system,
                    bullet_vs_redbrick_system,
                    bullet_vs_spawning_tank_system,
                    bullet_vs_enemy_system,
                    bullet_vs_player_system,
                    boom_effect,
                ),
                despawn_marked_entities,
            )
                .chain()
                .run_if(in_state(Screen::GamePlay).and(game_is_active)),
        )
        .add_systems(
            PostUpdate,
            enter_game_over_after_camp_boom.run_if(in_state(Screen::GamePlay)),
        );
}

/// 堡垒被摧毁时的爆炸效果，动画结束后进入 GameOver
#[derive(Component)]
struct CampBoomEffect;

/// 堡垒是否已被摧毁（防止重复触发爆炸）
#[derive(Resource, Default)]
struct CampDestroyPending(bool);

fn reset_camp_destroy_pending(mut pending: ResMut<CampDestroyPending>) {
    pending.0 = false;
}

/// 本帧已被其他墙体碰撞系统处理的炮弹（Commands 延迟应用，需同帧追踪）
#[derive(Resource, Default)]
struct BulletsConsumedThisFrame(HashSet<Entity>);

// ── 碰撞分发辅助 ──────────────────────────────────────────

/// 区分出子弹实体与目标实体，两发炮弹时返回 true
fn distinguish_entities(
    e1: Entity,
    e2: Entity,
    bullet_query: &Query<&BulletInfo>,
) -> (Entity, Entity, bool) {
    match (bullet_query.get(e1), bullet_query.get(e2)) {
        (Ok(_), Err(_)) => (e1, e2, false),
        (Err(_), Ok(_)) => (e2, e1, false),
        (Ok(_), Ok(_)) => (e1, e2, true),    // 两发炮弹碰撞
        (Err(_), Err(_)) => (e1, e2, false), // 都不是炮弹，按普通碰撞处理
    }
}

// ── 碰撞处理系统 ──────────────────────────────────────────

/// 炮弹 VS 炮弹：互相销毁
fn bullet_vs_bullet_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    bullet_query: Query<&BulletInfo>,
    mut consumed: ResMut<BulletsConsumedThisFrame>,
) {
    consumed.0.clear();
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet {
            consumed.0.insert(bullet_entity);
            consumed.0.insert(other_entity);
            commands.entity(bullet_entity).insert(ShouldDespawn);
            commands.entity(other_entity).insert(ShouldDespawn);
        }
    }
}

/// 炮弹 VS 堡垒：游戏结束
fn bullet_vs_camp_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut messages: MessageWriter<BoomMsg>,
    bullet_query: Query<&BulletInfo>,
    camp_query: Query<&Camp>,
    transform_query: Query<&Transform>,
    parent_query: Query<&ChildOf>,
    img_asset: Res<ImgAsset>,
    mut camp_destroy_pending: ResMut<CampDestroyPending>,
    mut consumed: ResMut<BulletsConsumedThisFrame>,
) {
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet || camp_query.get(other_entity).is_err() || camp_destroy_pending.0 {
            continue;
        }
        camp_destroy_pending.0 = true;
        consumed.0.insert(bullet_entity);
        commands.entity(bullet_entity).insert(ShouldDespawn);
        if let Ok(camp_transform) = transform_query.get(other_entity) {
            let mut boom_transform = camp_transform.clone();
            boom_transform.translation.x += 8.0;
            boom_transform.translation.y += 8.0;
            boom_transform.translation.z += 2.0;
            messages.write(BoomMsg {
                transform: boom_transform,
                boom_level: 4,
                show_prop: false,
            });
        }
        if let Ok(child_of) = parent_query.get(other_entity) {
            commands
                .entity(child_of.parent())
                .insert(TilemapTexture::Single(img_asset.camp_break.clone()));
        }
    }
}

/// 炮弹 VS 石墙
fn bullet_vs_stone_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut messages: MessageWriter<BoomMsg>,
    bullet_query: Query<&BulletInfo>,
    stone_query: Query<&Stone>,
    transform_query: Query<&Transform>,
    mut consumed: ResMut<BulletsConsumedThisFrame>,
) {
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet || stone_query.get(other_entity).is_err() {
            continue;
        }
        consumed.0.insert(bullet_entity);
        if let (Ok(bullet_transform), Ok(bullet_info)) = (
            transform_query.get(bullet_entity),
            bullet_query.get(bullet_entity),
        ) {
            let level = if bullet_info.level == 3 {
                commands.entity(other_entity).insert(ShouldDespawn);
                1
            } else {
                0
            };
            messages.write(BoomMsg {
                transform: bullet_transform.clone(),
                boom_level: level,
                show_prop: false,
            });
        }
        commands.entity(bullet_entity).insert(ShouldDespawn);
    }
}

/// 炮弹 VS 钢铁边界
fn bullet_vs_steel_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut messages: MessageWriter<BoomMsg>,
    bullet_query: Query<&BulletInfo>,
    steel_query: Query<&Steel>,
    transform_query: Query<&Transform>,
    mut consumed: ResMut<BulletsConsumedThisFrame>,
) {
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet || steel_query.get(other_entity).is_err() {
            continue;
        }
        consumed.0.insert(bullet_entity);
        if let Ok(bullet_transform) = transform_query.get(bullet_entity) {
            messages.write(BoomMsg {
                transform: bullet_transform.clone(),
                boom_level: 0,
                show_prop: false,
            });
        }
        commands.entity(bullet_entity).insert(ShouldDespawn);
    }
}

/// 炮弹 VS 红砖墙
fn bullet_vs_redbrick_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut messages: MessageWriter<BoomMsg>,
    bullet_query: Query<&BulletInfo>,
    redbrick_query: Query<&RedBrick>,
    hidden_redbrick_query: Query<(), With<ShovelHiddenRedBrick>>,
    consumed: Res<BulletsConsumedThisFrame>,
    transform_query: Query<&Transform>,
) {
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet
            || consumed.0.contains(&bullet_entity)
            || hidden_redbrick_query.get(other_entity).is_ok()
            || redbrick_query.get(other_entity).is_err()
        {
            continue;
        }
        if let Ok(bullet_transform) = transform_query.get(bullet_entity) {
            messages.write(BoomMsg {
                transform: bullet_transform.clone(),
                boom_level: 1,
                show_prop: false,
            });
        }
        commands.entity(other_entity).insert(ShouldDespawn);
        commands.entity(bullet_entity).insert(ShouldDespawn);
    }
}

/// 炮弹 VS 生成动画坦克：直接销毁炮弹
fn bullet_vs_spawning_tank_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    bullet_query: Query<&BulletInfo>,
    marked_bullets: Query<Entity, With<ShouldDespawn>>,
    spawning_tank_query: Query<Entity, With<SpawnAnimation>>,
) {
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet
            || marked_bullets.get(bullet_entity).is_ok()
            || spawning_tank_query.get(other_entity).is_err()
        {
            continue;
        }
        commands.entity(bullet_entity).insert(ShouldDespawn);
    }
}

/// 炮弹 VS 敌人
fn bullet_vs_enemy_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut messages: MessageWriter<BoomMsg>,
    mut prop_message: MessageWriter<PropsSpawn>,
    bullet_query: Query<&BulletInfo>,
    mut enemy_query: Query<(&mut EnemyInfo, &mut MoveAnimation, &EnemyAI)>,
    transform_query: Query<&Transform>,
    img_asset: Res<ImgAsset>,
    mut enemy_number_state: ResMut<EnemyNumberState>,
    mut player_query: Query<&mut PlayerInfo>,
    children: Query<&Children>,
    bulletproof_query: Query<(), With<BulletproofLayer>>,
    waterproofer_query: Query<(), With<Waterproofer>>,
) {
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet || enemy_query.get(other_entity).is_err() {
            continue;
        }
        let bullet_info = match bullet_query.get(bullet_entity) {
            Ok(info) => info,
            Err(_) => continue,
        };
        if bullet_info.horde == 1 {
            // 玩家命中敌坦克
            if let Ok((mut enemy_info, mut move_animation, ai)) = enemy_query.get_mut(other_entity) {
                if player_has_bulletproof(other_entity, &children, &bulletproof_query) {
                    commands.entity(bullet_entity).insert(ShouldDespawn);
                    continue;
                }
                if player_has_waterproofer(other_entity, &children, &waterproofer_query) {
                    if let Ok(enemy_children) = children.get(other_entity) {
                        for child in enemy_children.iter() {
                            if waterproofer_query.get(child).is_ok() {
                                commands.entity(child).despawn();
                                break;
                            }
                        }
                    }
                    commands.entity(bullet_entity).insert(ShouldDespawn);
                    continue;
                }
                if enemy_info.carrying_prop {
                    // 携带道具，发送道具生成消息
                    prop_message.write(PropsSpawn);
                }
                if enemy_info.life == 1 {
                    // 最后一命 → 摧毁
                    if let Ok(enemy_transform) = transform_query.get(other_entity) {
                        let mut boom_transform = enemy_transform.clone();
                        boom_transform.translation.z += 2.0;
                        messages.write(BoomMsg {
                            transform: boom_transform,
                            boom_level: 2,
                            show_prop: enemy_info.carrying_prop,
                        });
                    }
                    commands.entity(other_entity).insert(ShouldDespawn);
                    commands.entity(bullet_entity).insert(ShouldDespawn);
                    enemy_number_state.current_enemy_count -= 1;
                    if let Ok(mut player_info) = player_query.get_mut(bullet_info.entity) {
                        player_info.score += enemy_info.score;
                        player_info
                            .enemy_kills
                            .record(ai.tank_type, enemy_info.score);
                    }
                } else {
                    // 生命减1，更新精灵图
                    enemy_info.life -= 1;
                    move_animation.frames = match enemy_info.life {
                        2 => vec![img_asset.e3_1_0.clone(), img_asset.e3_1_1.clone()],
                        _ => vec![img_asset.e3_0_0.clone(), img_asset.e3_0_1.clone()],
                    };
                    if let Ok(enemy_transform) = transform_query.get(other_entity) {
                        let mut boom_transform = enemy_transform.clone();
                        boom_transform.translation.z += 2.0;
                        messages.write(BoomMsg {
                            transform: boom_transform,
                            boom_level: 0,
                            show_prop: enemy_info.carrying_prop,
                        });
                    }
                    commands.entity(bullet_entity).insert(ShouldDespawn);
                }
            }
        } else {
            // 敌坦克命中敌坦克
            commands.entity(bullet_entity).insert(ShouldDespawn);
            if let Ok(bullet_transform) = transform_query.get(bullet_entity) {
                messages.write(BoomMsg {
                    transform: bullet_transform.clone(),
                    boom_level: 0,
                    show_prop: false,
                });
            }
        }
    }
}

/// 玩家是否拥有防弹层（防弹层为玩家子实体）
fn player_has_bulletproof(
    player: Entity,
    children: &Query<&Children>,
    bulletproof_query: &Query<(), With<BulletproofLayer>>,
) -> bool {
    children.get(player).is_ok_and(|children| {
        children
            .iter()
            .any(|child| bulletproof_query.get(child).is_ok())
    })
}

/// 玩家是否拥有防水层（防水层为玩家子实体）
fn player_has_waterproofer(
    player: Entity,
    children: &Query<&Children>,
    waterproofer_query: &Query<(), With<Waterproofer>>,
) -> bool {
    children.get(player).is_ok_and(|children| {
        children
            .iter()
            .any(|child| waterproofer_query.get(child).is_ok())
    })
}

/// 炮弹 VS 玩家
fn bullet_vs_player_system(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut messages: MessageWriter<BoomMsg>,
    mut spawn_requests: MessageWriter<PlayerSpawnRequest>,
    bullet_query: Query<&BulletInfo>,
    marked_bullets: Query<Entity, With<ShouldDespawn>>,
    mut player_query: Query<&mut PlayerInfo, With<Collider>>,
    children: Query<&Children>,
    bulletproof_query: Query<(), With<BulletproofLayer>>,
    waterproofer_query: Query<(), With<Waterproofer>>,
    transform_query: Query<&Transform>,
) {
    for event in collision_reader.read() {
        let (bullet_entity, other_entity, is_all_bullet) =
            distinguish_entities(event.collider1, event.collider2, &bullet_query);
        if is_all_bullet
            || marked_bullets.get(bullet_entity).is_ok()
            || player_query.get(other_entity).is_err()
        {
            continue;
        }
        if player_has_bulletproof(other_entity, &children, &bulletproof_query) {
            commands.entity(bullet_entity).insert(ShouldDespawn);
            continue;
        }
        if player_has_waterproofer(other_entity, &children, &waterproofer_query) {
            // 有防水层 → 销毁防水层和炮弹
            if let Ok(player_children) = children.get(other_entity) {
                for child in player_children.iter() {
                    if waterproofer_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                        break;
                    }
                }
            }
            commands.entity(bullet_entity).insert(ShouldDespawn);
            continue;
        }
        let bullet_info = match bullet_query.get(bullet_entity) {
            Ok(info) => info,
            Err(_) => continue,
        };
        if bullet_info.horde == 1 {
            // 玩家命中玩家 → 闪烁僵直
            if let Ok(bullet_transform) = transform_query.get(bullet_entity) {
                messages.write(BoomMsg {
                    transform: bullet_transform.clone(),
                    boom_level: 0,
                    show_prop: false,
                });
            }
            commands.entity(bullet_entity).insert(ShouldDespawn);
            commands.entity(other_entity).insert(PlayerIdle::default());
        } else {
            let Ok(mut player_info) = player_query.get_mut(other_entity) else {
                continue;
            };
            commands.entity(bullet_entity).insert(ShouldDespawn);
            // 同一帧内可能收到多次碰撞事件，已摧毁则跳过
            if player_info.life == 0 {
                continue;
            }

            player_info.life -= 1;

            if player_info.life == 0 {
                // 敌坦克命中玩家 → 生命降到0 摧毁
                player_info.chance -= 1;
                let player_id = player_info.id;
                let remaining_chance = player_info.chance;
                if let Ok(player_transform) = transform_query.get(other_entity) {
                    let mut boom_transform = player_transform.clone();
                    boom_transform.translation.z += 2.0;
                    messages.write(BoomMsg {
                        transform: boom_transform,
                        boom_level: 3,
                        show_prop: false,
                    });
                }
                strip_tank_physics(&mut commands, other_entity);
                commands
                    .entity(other_entity)
                    .remove::<(Sprite, Transform, Facing, MoveAnimation, PlayerIdle)>();
                if remaining_chance > 0 {
                    spawn_requests.write(PlayerSpawnRequest {
                        entity: other_entity,
                        id: player_id,
                    });
                }
            }
        }
    }
}

// ── 公共系统 ──────────────────────────────────────────────

/// 统一销毁所有被标记的实体
fn despawn_marked_entities(
    mut commands: Commands,
    marked_query: Query<Entity, With<ShouldDespawn>>,
) {
    for entity in marked_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 爆炸效果
fn boom_effect(
    mut commands: Commands,
    mut messages: MessageReader<BoomMsg>,
    img_asset: Res<ImgAsset>,
    audio_asset: Res<AudioAsset>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for message in messages.read() {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let time = match message.boom_level {
            4 => 0.1,  // 堡垒被摧毁
            3 => 0.12, // 玩家被摧毁
            2 => 0.15, // 敌坦克被摧毁
            1 => 0.13, // 石墙/红砖被摧毁
            0 => 0.14, // 石墙/钢铁边界未被摧毁
            _ => 0.1,  // 默认
        };

        let effect_animation = EffectAnimation {
            timer: Timer::from_seconds(time, TimerMode::Repeating),
            first: 0,
            last: message.boom_level,
        };
        let mut transform = message.transform.clone();
        transform.translation.z += 1.;
        let mut entity = commands.spawn((
            Sprite::from_atlas_image(
                img_asset.boom.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: effect_animation.first,
                },
            ),
            transform,
            effect_animation,
        ));
        if message.boom_level == 4 {
            entity.insert(CampBoomEffect);
        }
        // 播放音效
        let audio_effect = match message.boom_level {
            4 | 3 => audio_asset.big_explosion.clone(), // 玩家堡垒摧毁
            2 => {
                let audio = if message.show_prop {
                    audio_asset.powerup_appear.clone()
                } else {
                    audio_asset.bullet_explosion.clone()
                };
                audio
            } // 敌坦克摧毁
            1 => audio_asset.bullet_hit_2.clone(),      // 命中石墙/红砖摧毁
            0 => {
                let audio = if message.show_prop {
                    audio_asset.prop_show.clone()
                } else {
                    audio_asset.bullet_hit_1.clone()
                };
                audio
            } // 命中石墙/钢铁边界为摧毁
            _ => audio_asset.bullet_hit_2.clone(),      // 默认
        };
        commands.spawn(sound_effect(audio_effect));
    }
}

/// 堡垒爆炸动画结束后进入 GameOver
fn enter_game_over_after_camp_boom(
    mut removed: RemovedComponents<CampBoomEffect>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    if removed.read().next().is_some() {
        next_state.set(Screen::GameOver);
    }
}
