use avian2d::{collision::hooks::ActiveCollisionHooks, dynamics::rigid_body::LinearVelocity};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

use crate::{
    audio::{music, sound_effect},
    collision::add_bullet_collision,
    common_component::{BulletInfo, Facing, MoveAnimation, ROTATION, facing_from_velocity},
    config::*,
    input_manager::PlayerAction,
    map::{IceTiles, apply_ice_movement},
    player::{PlayerIdle, PlayerInfo},
    resource_manage::{AudioAsset, ImgAsset},
    screens::{game_is_active, Screen},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (move_player, fire).run_if(in_state(Screen::GamePlay).and(game_is_active)),
    );
}

/// 玩家控制
fn move_player(
    mut commands: Commands,
    time: Res<Time>,
    ice_tiles: Option<Res<IceTiles>>,
    mut player_query: Query<(
        &mut LinearVelocity,
        &mut Facing,
        &mut Transform,
        &mut MoveAnimation,
        &mut PlayerInfo,
        &ActionState<PlayerAction>,
        Option<&PlayerIdle>,
    )>,
    audio_asset: Res<AudioAsset>,
) {
    for (
        mut velocity,
        mut facing,
        mut transform,
        mut animation,
        mut player_info,
        action_state,
        idle,
    ) in player_query.iter_mut()
    {
        // 僵直状态：停止移动
        if idle.is_some() {
            animation.playing = false;
            velocity.0 = Vec2::ZERO;
            if let Some(sound) = player_info.move_sound {
                commands.entity(sound).despawn();
                player_info.move_sound = None;
            }
            continue;
        }
        let mut direction = Vec2::ZERO;
        if action_state.pressed(&PlayerAction::MoveUp) {
            direction.y += 1.;
            *facing = Facing::Up;
        } else if action_state.pressed(&PlayerAction::MoveDown) {
            direction.y -= 1.;
            *facing = Facing::Down;
        } else if action_state.pressed(&PlayerAction::MoveLeft) {
            direction.x -= 1.;
            *facing = Facing::Left;
        } else if action_state.pressed(&PlayerAction::MoveRight) {
            direction.x += 1.;
            *facing = Facing::Right
        }

        let on_ice = ice_tiles
            .as_ref()
            .is_some_and(|tiles| tiles.contains(transform.translation));
        let target_velocity = if direction != Vec2::ZERO {
            direction * PLAYER_MOVE_SPEED
        } else {
            Vec2::ZERO
        };

        velocity.0 = if on_ice {
            apply_ice_movement(velocity.0, target_velocity, time.delta_secs())
        } else {
            target_velocity
        };

        let is_moving = velocity.0.length_squared() > ICE_MIN_SPEED * ICE_MIN_SPEED;
        if direction != Vec2::ZERO {
            animation.playing = true;
            transform.rotation = Quat::from_rotation_z(ROTATION[*facing as usize]);
            if player_info.move_sound.is_none() {
                let sound = commands.spawn(music(audio_asset.player_move.clone())).id();
                player_info.move_sound = Some(sound);
            }
        } else if is_moving {
            animation.playing = true;
            *facing = facing_from_velocity(velocity.0);
            transform.rotation = Quat::from_rotation_z(ROTATION[*facing as usize]);
            if player_info.move_sound.is_none() {
                let sound = commands.spawn(music(audio_asset.player_move.clone())).id();
                player_info.move_sound = Some(sound);
            }
        } else {
            animation.playing = false;
            if let Some(sound) = player_info.move_sound {
                commands.entity(sound).despawn();
                player_info.move_sound = None;
            }
        }
    }
}

/// 开火
fn fire(
    mut commands: Commands,
    img_asset: Res<ImgAsset>,
    player_query: Query<(
        Entity,
        &PlayerInfo,
        &Facing,
        &Transform,
        &ActionState<PlayerAction>,
        Option<&PlayerIdle>,
    )>,
    bullet_query: Query<&BulletInfo>,
    audio_asset: Res<AudioAsset>,
) {
    for (entity, player_info, facing, transform, action_state, idle) in player_query.iter() {
        // 僵直状态：不能开火
        if idle.is_some() {
            continue;
        }
        if action_state.just_pressed(&PlayerAction::Fire) {
            // 计算当前屏幕上该玩家的活跃炮弹数
            let active_count = bullet_query
                .iter()
                .filter(|info| info.entity == entity && info.horde == 1)
                .count() as u8;
            // 最大炮弹数：等级 >= 2 时最多 2 发，否则 1 发
            let max_bullets: u8 = if player_info.level >= 2 { 2 } else { 1 };
            let to_fire = max_bullets.saturating_sub(active_count);
            if to_fire == 0 {
                continue;
            }

            // 播放开火音效（无论几发只播一次）
            commands.spawn(sound_effect(audio_asset.player_fire.clone()));

            // 根据玩家等级设置炮弹速度倍率
            let speed_ratio = match player_info.level {
                0 => PLAYER_BULLET_SPEED,
                1 => PLAYER_BULLET_SPEED_1,
                2 => PLAYER_BULLET_SPEED_2,
                3 => PLAYER_BULLET_SPEED_3,
                _ => PLAYER_BULLET_SPEED,
            };

            for i in 0..to_fire {
                spawn_player_bullet(
                    &mut commands,
                    &img_asset,
                    entity,
                    player_info.level,
                    facing,
                    transform,
                    speed_ratio,
                    i, // 第几发（0=第一发，1=第二发偏移位置）
                );
            }
        }
    }
}

/// 生成一发玩家炮弹
fn spawn_player_bullet(
    commands: &mut Commands,
    img_asset: &ImgAsset,
    player_entity: Entity,
    level: u8,
    facing: &Facing,
    transform: &Transform,
    speed_ratio: f32,
    index: u8,
) {
    // 第二发炮弹稍微靠后偏移
    let behind = if index > 0 { 6.0 } else { 0.0 };
    let fire_position = match facing {
        Facing::Up => Vec3::new(
            transform.translation.x,
            transform.translation.y + 5.0 - behind,
            1.0,
        ),
        Facing::Down => Vec3::new(
            transform.translation.x,
            transform.translation.y - 5.0 + behind,
            1.0,
        ),
        Facing::Left => Vec3::new(
            transform.translation.x - 5.0 + behind,
            transform.translation.y,
            1.0,
        ),
        Facing::Right => Vec3::new(
            transform.translation.x + 5.0 - behind,
            transform.translation.y,
            1.0,
        ),
    };

    let velocity = match facing {
        Facing::Up => Vec2::new(0., 100.),
        Facing::Down => Vec2::new(0., -100.),
        Facing::Left => Vec2::new(-100., 0.),
        Facing::Right => Vec2::new(100., 0.),
    };

    commands.spawn((
        BulletInfo {
            entity: player_entity,
            level,
            horde: 1,
        },
        Sprite {
            image: img_asset.bullet.clone(),
            ..default()
        },
        ActiveCollisionHooks::FILTER_PAIRS,
        add_bullet_collision(velocity * speed_ratio),
        Transform {
            translation: fire_position,
            rotation: Quat::from_rotation_z(ROTATION[*facing as usize]),
            ..default()
        },
    ));
}
