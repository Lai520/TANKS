use avian2d::collision::{collider::Collider, collision_events::CollisionStart};
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    audio::sound_effect,
    collision::{BoomMsg, strip_tank_physics},
    common_component::{
        Facing, MoveAnimation, add_bulletproof, add_waterproofer, enemy_loop_animation,
    },
    config::{NO_INVINCIBLE_TIME_OF_PROTECT, PICK_UP_TIME},
    enemy::{EnemyAI, EnemyInfo, EnemyNumberState},
    map::{Camp, RedBrick, ShovelHiddenRedBrick, Stone},
    player::{PlayerIdle, PlayerInfo, PlayerSpawnRequest},
    props::{
        PropType, distinguish_prop_entities, prop_generate::Prop, shovel_effect,
    },
    resource_manage::{AudioAsset, ImgAsset},
    screens::{game_is_active, Screen},
};

/// 敌人拾取工兵铲相关查询
#[derive(SystemParam)]
struct EnemyShovelParams<'w, 's> {
    camp_query: Query<'w, 's, &'static GlobalTransform, With<Camp>>,
    red_brick_query: Query<'w, 's, (Entity, &'static GlobalTransform), With<RedBrick>>,
    hidden_red_brick_query:
        Query<'w, 's, (Entity, &'static GlobalTransform), With<ShovelHiddenRedBrick>>,
    stone_query: Query<'w, 's, (Entity, &'static GlobalTransform), With<Stone>>,
    camp_shovel_effect: ResMut<'w, shovel_effect::CampShovelEffect>,
    camp_ring_cleared: ResMut<'w, shovel_effect::CampRingWallsCleared>,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        enemy_grab_power_up.run_if(in_state(Screen::GamePlay).and(game_is_active)),
    );
}

fn enemy_grab_power_up(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut enemy_query: Query<(&mut EnemyInfo, &EnemyAI), With<Collider>>,
    mut player_tank_query: Query<(Entity, &mut PlayerInfo, &Transform), (With<PlayerInfo>, With<Collider>)>,
    prop_query: Query<&Prop>,
    audio_asset: Res<AudioAsset>,
    mut messages: MessageWriter<BoomMsg>,
    mut spawn_requests: MessageWriter<PlayerSpawnRequest>,
    mut enemy_number_state: ResMut<EnemyNumberState>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    img_asset: Res<ImgAsset>,
    mut shovel_params: EnemyShovelParams,
) {
    for event in collision_reader.read() {
        let (prop_entity, other_entity, valid) =
            distinguish_prop_entities(event.collider1, event.collider2, &prop_query);
        if !valid {
            continue;
        }
        let Ok(prop) = prop_query.get(prop_entity) else {
            continue;
        };
        let Ok((mut enemy, ai)) = enemy_query.get_mut(other_entity) else {
            continue;
        };
        commands.spawn(sound_effect(audio_asset.powerup_pick.clone()));
        match prop.0 {
            PropType::Boom => {
                for (player_entity, mut player_info, transform) in player_tank_query.iter_mut() {
                    if player_info.life == 0 {
                        continue;
                    }
                    let mut boom_transform = transform.clone();
                    boom_transform.translation.z += 2.0;
                    messages.write(BoomMsg {
                        transform: boom_transform,
                        boom_level: 3,
                        show_prop: false,
                    });
                    strip_tank_physics(&mut commands, player_entity);
                    commands.entity(player_entity).remove::<(
                        Sprite,
                        Transform,
                        Facing,
                        MoveAnimation,
                        PlayerIdle,
                    )>();
                    player_info.life = 0;
                    if player_info.chance > 0 {
                        player_info.chance -= 1;
                    }
                    let remaining_chance = player_info.chance;
                    let player_id = player_info.id;
                    if remaining_chance > 0 {
                        spawn_requests.write(PlayerSpawnRequest {
                            entity: player_entity,
                            id: player_id,
                        });
                    }
                }
            }
            PropType::Shovel => {
                shovel_effect::destroy_camp_ring_permanently(
                    &mut commands,
                    &mut shovel_params.camp_shovel_effect,
                    &mut shovel_params.camp_ring_cleared,
                    &shovel_params.camp_query,
                    &shovel_params.red_brick_query,
                    &shovel_params.hidden_red_brick_query,
                    &shovel_params.stone_query,
                );
            }
            PropType::Idle => {
                for (player_entity, _, _) in player_tank_query.iter() {
                    commands.entity(player_entity).insert(PlayerIdle {
                        timer: Timer::from_seconds(PICK_UP_TIME, TimerMode::Once),
                        flash_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    });
                }
            }
            PropType::Star | PropType::Gun => {
                if prop.0 == PropType::Gun {
                    enemy.level = 3;
                } else if enemy.level < 3 {
                    enemy.level += 1;
                }
                let move_animation =
                    enemy_loop_animation(ai.tank_type, enemy.carrying_prop, &img_asset);
                commands.entity(other_entity).insert((
                    Sprite {
                        image: move_animation.frames[0].clone(),
                        ..default()
                    },
                    move_animation,
                ));
            }
            PropType::Bulletproof => {
                let waterproof = commands.spawn(add_waterproofer(&img_asset)).id();
                let bulletproof = commands
                    .spawn(add_bulletproof(
                        &img_asset,
                        &mut texture_atlas_layouts,
                        NO_INVINCIBLE_TIME_OF_PROTECT,
                    ))
                    .id();
                commands
                    .entity(other_entity)
                    .add_children(&[waterproof, bulletproof]);
            }
            PropType::Chance => {
                enemy_number_state.max_enemy_count += 1;
            }
            PropType::Waterproof => {
                let waterproof = commands.spawn(add_waterproofer(&img_asset)).id();
                commands.entity(other_entity).add_child(waterproof);
            }
        }
        commands.entity(prop_entity).despawn();
    }
}
