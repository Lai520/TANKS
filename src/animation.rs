use bevy::prelude::*;

use crate::common_component::{
    BulletproofLayer, EffectAnimation, EnemyTankSpawn, MoveAnimation, PlayerTankSpawn,
    SpawnAnimation, TankId,
};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<EnemyTankSpawn>()
        .add_message::<PlayerTankSpawn>()
        .add_systems(
            Update,
            (
                move_animation,
                effect_animation,
                tank_spawn_animation,
                bulletproof_animation,
            ),
        );
}

/// 移动动画
fn move_animation(time: Res<Time>, mut animation_query: Query<(&mut MoveAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut animation_query {
        if !animation.playing {
            continue;
        }

        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if animation.cur_frames < animation.frames.len() {
                *sprite = animation.frames[animation.cur_frames].clone().into();
                animation.cur_frames += 1;
            } else {
                animation.cur_frames = 0;
            }
        }
    }
}

/// 效果动画
fn effect_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_animation: Query<(&mut EffectAnimation, &mut Sprite, Entity)>,
) {
    for (mut indices, mut sprite, entity) in &mut effect_animation {
        indices.timer.tick(time.delta());

        if indices.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index == indices.last {
                commands.entity(entity).despawn();
            } else {
                atlas.index += 1;
            }
        }
    }
}

/// 坦克生成动画（供 enemy 模块保证执行顺序）
pub(crate) fn tank_spawn_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_animation_query: Query<(&mut SpawnAnimation, &TankId, Entity, &mut Sprite)>,
    mut enemy_spawn: MessageWriter<EnemyTankSpawn>,
    mut player_spawn: MessageWriter<PlayerTankSpawn>,
) {
    for (mut spawn_animation, tank_id, entity, mut sprite) in &mut spawn_animation_query {
        spawn_animation.timer.tick(time.delta());

        if spawn_animation.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index == spawn_animation.last {
                spawn_animation.loop_num -= 1;
                if spawn_animation.loop_num == 0 {
                    if tank_id.0 == 0 {
                        enemy_spawn.write(EnemyTankSpawn {
                            entity,
                            transform: spawn_animation.transform,
                        });
                    } else {
                        player_spawn.write(PlayerTankSpawn {
                            transform: spawn_animation.transform,
                            id: tank_id.0,
                        });
                        commands.entity(entity).despawn();
                    }
                } else {
                    atlas.index = spawn_animation.first;
                }
            } else {
                atlas.index += 1;
            }
        }
    }
}

/// 防弹层动画
fn bulletproof_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut bulletproof_query: Query<(Entity, &mut BulletproofLayer, &mut Sprite)>,
) {
    for (entity, mut bulletproof, mut sprite) in &mut bulletproof_query {
        bulletproof.timer.tick(time.delta());

        if bulletproof.timer.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        bulletproof.frames_timer.tick(time.delta());

        if bulletproof.frames_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if bulletproof.cur_frames < bulletproof.last {
                bulletproof.cur_frames += 1;
                atlas.index = bulletproof.cur_frames;
            } else {
                bulletproof.cur_frames = bulletproof.first;
                atlas.index = bulletproof.first;
            }
        }
    }
}
