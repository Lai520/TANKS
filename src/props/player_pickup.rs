use avian2d::collision::{collider::Collider, collision_events::CollisionStart};
use bevy::prelude::*;

use crate::{
    audio::sound_effect,
    collision::{BoomMsg, ShouldDespawn},
    common_component::{add_bulletproof, add_waterproofer, player_loop_animation},
    config::{NO_INVINCIBLE_TIME_OF_PROTECT, PICK_UP_TIME},
    enemy::{EnemyInfo, EnemyNumberState},
    map::{Camp, RedBrick, Stone},
    player::PlayerInfo,
    props::{PropStatus, PropType, distinguish_prop_entities, prop_generate::Prop, shovel_effect},
    resource_manage::{AudioAsset, ImgAsset},
    screens::{Screen, game_is_active},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        player_grab_power_up.run_if(in_state(Screen::GamePlay).and(game_is_active)),
    );
}

fn player_grab_power_up(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut player_query: Query<&mut PlayerInfo, With<Collider>>,
    prop_query: Query<&Prop>,
    audio_asset: Res<AudioAsset>,
    enemy_query: Query<(Entity, &Transform), With<EnemyInfo>>,
    mut messages: MessageWriter<BoomMsg>,
    mut enemy_number_state: ResMut<EnemyNumberState>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    img_asset: Res<ImgAsset>,
    camp_query: Query<&GlobalTransform, With<Camp>>,
    red_brick_query: Query<(Entity, &GlobalTransform), With<RedBrick>>,
    stone_tilemap_query: Query<&bevy_ecs_tilemap::map::TilemapId, With<Stone>>,
    tilemap_texture_query: Query<&bevy_ecs_tilemap::map::TilemapTexture>,
    mut camp_shovel_effect: ResMut<shovel_effect::CampShovelEffect>,
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
        let Ok(mut player) = player_query.get_mut(other_entity) else {
            continue;
        };
        if prop.0 == PropType::Chance {
            commands.spawn(sound_effect(audio_asset.add_life.clone()));
        } else {
            commands.spawn(sound_effect(audio_asset.powerup_pick.clone()));
        }
        match prop.0 {
            PropType::Chance => {
                player.chance += 1;
            }
            PropType::Idle => {
                for (enemy, _) in enemy_query.iter() {
                    let prop_status = commands
                        .spawn(PropStatus {
                            timer: Timer::from_seconds(PICK_UP_TIME, TimerMode::Once),
                            prop_type: prop.0,
                        })
                        .id();
                    commands.entity(enemy).add_child(prop_status);
                }
            }
            PropType::Shovel => {
                shovel_effect::activate_camp_shovel_effect(
                    &mut commands,
                    &mut camp_shovel_effect,
                    &camp_query,
                    &red_brick_query,
                    &stone_tilemap_query,
                    &tilemap_texture_query,
                );
            }
            PropType::Boom => {
                for (enemy, transform) in enemy_query.iter() {
                    messages.write(BoomMsg {
                        transform: transform.clone(),
                        boom_level: 2,
                        show_prop: false,
                    });
                    commands.entity(enemy).insert(ShouldDespawn);
                    enemy_number_state.current_enemy_count -= 1;
                }
            }
            PropType::Star | PropType::Gun => {
                if prop.0 == PropType::Gun {
                    player.level = 3;
                }
                if player.level < 3 {
                    player.level += 1;
                }
                let move_animation = player_loop_animation(player.id, player.level, &img_asset);
                commands.entity(other_entity).insert((
                    Sprite {
                        image: move_animation.frames[0].clone(),
                        ..default()
                    },
                    move_animation,
                ));
            }
            PropType::Bulletproof => {
                let bulletproof_entity = commands
                    .spawn(add_bulletproof(
                        &img_asset,
                        &mut texture_atlas_layouts,
                        NO_INVINCIBLE_TIME_OF_PROTECT,
                    ))
                    .id();
                commands.entity(other_entity).add_child(bulletproof_entity);
            }
            PropType::Waterproof => {
                let waterproofet = commands.spawn(add_waterproofer(&img_asset)).id();
                commands.entity(other_entity).add_child(waterproofet);
            }
        }
        commands.entity(prop_entity).despawn();
    }
}
