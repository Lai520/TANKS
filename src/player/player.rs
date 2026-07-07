use bevy::prelude::*;
use leafwing_input_manager::input_map::InputMap;

use crate::{
    collision::add_tank_collision,
    common_component::{
        Facing, PlayerTankSpawn, add_bulletproof, player_loop_animation, spawn_animation,
    },
    config::{NO_INVINCIBLE_TIME, PLAYER_CHANCE, TANK_RENDER_Z},
    input_manager::PlayerAction,
    map::{MapState, PlayerSpawnPos},
    player::{EnemyKillRecords, PlayerInfo, PlayerSpawnRequest, PlayerSpawning},
    resource_manage::ImgAsset,
    screens::{PlayerNumber, Screen},
};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<PlayerSpawnRequest>()
        .add_systems(OnEnter(Screen::GamePlay), spwan_player_info)
        .add_systems(
            OnEnter(MapState::Complete),
            request_initial_player_spawn.run_if(in_state(Screen::GamePlay)),
        )
        .add_systems(
            Update,
            spawn_player.run_if(in_state(MapState::Complete).and(in_state(Screen::GamePlay))),
        )
        .add_systems(
            PostUpdate,
            play_player_spawn_animation
                .run_if(in_state(MapState::Complete).and(in_state(Screen::GamePlay))),
        );
}

/// 添加玩家信息
fn spwan_player_info(mut commands: Commands, player_number: Res<PlayerNumber>) {
    for i in 0..player_number.value {
        commands.spawn((
            Visibility::Visible,
            PlayerInfo {
                id: i + 1,
                life: 1,
                level: 0,
                score: 0,
                enemy_kills: EnemyKillRecords::default(),
                chance: PLAYER_CHANCE,
                move_sound: None,
            },
        ));
    }
}

/// 首次进入游戏且地图加载完成，请求生成玩家
fn request_initial_player_spawn(
    player_number: Res<PlayerNumber>,
    player_query: Query<(Entity, &PlayerInfo)>,
    mut requests: MessageWriter<PlayerSpawnRequest>,
) {
    for (entity, player_info) in &player_query {
        if player_info.id <= player_number.value && player_info.chance > 0 {
            requests.write(PlayerSpawnRequest {
                entity,
                id: player_info.id,
            });
        }
    }
}

/// 响应生成请求，播放玩家出生动画
fn play_player_spawn_animation(
    mut commands: Commands,
    mut requests: MessageReader<PlayerSpawnRequest>,
    player_spawn_pos_query: Query<(&PlayerSpawnPos, &Transform)>,
    player_query: Query<(Entity, &PlayerInfo)>,
    spawning_query: Query<(), With<PlayerSpawning>>,
    img_asset: Res<ImgAsset>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for request in requests.read() {
        if spawning_query.get(request.entity).is_ok() {
            continue;
        }

        let Ok((entity, player_info)) = player_query.get(request.entity) else {
            continue;
        };
        if entity != request.entity || player_info.id != request.id || player_info.chance == 0 {
            continue;
        }

        let Some((_, transform)) = player_spawn_pos_query
            .iter()
            .find(|(spawn_pos, _)| spawn_pos.0 == player_info.id)
        else {
            continue;
        };

        let animation_bundle = spawn_animation(
            &img_asset,
            &mut texture_atlas_layouts,
            *transform,
            player_info.id,
        );
        commands.spawn(animation_bundle);
        commands.entity(request.entity).insert(PlayerSpawning);
    }
}

/// 动画结束后生成玩家坦克
fn spawn_player(
    mut commands: Commands,
    img_asset: Res<ImgAsset>,
    mut messages: MessageReader<PlayerTankSpawn>,
    mut player_query: Query<(&mut PlayerInfo, Entity)>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for message in messages.read() {
        for (mut player, entity) in &mut player_query {
            if player.id == message.id {
                let gamepad = commands.spawn(()).id();
                let input_controller = set_controller(message.id, gamepad);
                let move_animation = player_loop_animation(message.id, 0, &img_asset);
                let first_frame = move_animation.frames[0].clone();
                player.life = 1;
                // 添加防弹层
                let bulletproof_entity = commands
                    .spawn(add_bulletproof(
                        &img_asset,
                        &mut texture_atlas_layouts,
                        NO_INVINCIBLE_TIME,
                    ))
                    .id();
                let mut tank_transform = message.transform;
                tank_transform.translation.z = TANK_RENDER_Z;
                commands
                    .entity(entity)
                    .try_insert(input_controller)
                    .try_insert(add_tank_collision("player"))
                    .insert(tank_transform)
                    .insert((
                        Sprite {
                            image: first_frame,
                            ..default()
                        },
                        Facing::Up,
                        move_animation,
                    ))
                    .add_child(bulletproof_entity)
                    .remove::<PlayerSpawning>();
            }
        }
    }
}

/// 添加控制器
fn set_controller(id: u8, gamepad: Entity) -> InputMap<PlayerAction> {
    if id == 1 {
        InputMap::new([
            (PlayerAction::MoveUp, KeyCode::KeyW),
            (PlayerAction::MoveDown, KeyCode::KeyS),
            (PlayerAction::MoveLeft, KeyCode::KeyA),
            (PlayerAction::MoveRight, KeyCode::KeyD),
            (PlayerAction::Fire, KeyCode::KeyJ),
            (PlayerAction::Pause, KeyCode::KeyP),
            (PlayerAction::Start, KeyCode::KeyO),
        ])
        .with(PlayerAction::MoveUp, GamepadButton::DPadUp)
        .with(PlayerAction::MoveDown, GamepadButton::DPadDown)
        .with(PlayerAction::MoveLeft, GamepadButton::DPadLeft)
        .with(PlayerAction::MoveRight, GamepadButton::DPadRight)
        .with(PlayerAction::Fire, GamepadButton::South)
        .with(PlayerAction::Start, GamepadButton::Start)
        .with(PlayerAction::Pause, GamepadButton::Select)
        .with_gamepad(gamepad)
    } else {
        InputMap::new([
            (PlayerAction::MoveUp, KeyCode::ArrowUp),
            (PlayerAction::MoveDown, KeyCode::ArrowDown),
            (PlayerAction::MoveLeft, KeyCode::ArrowLeft),
            (PlayerAction::MoveRight, KeyCode::ArrowRight),
            (PlayerAction::Fire, KeyCode::Numpad0),
        ])
        .with(PlayerAction::MoveUp, GamepadButton::DPadUp)
        .with(PlayerAction::MoveDown, GamepadButton::DPadDown)
        .with(PlayerAction::MoveLeft, GamepadButton::DPadLeft)
        .with(PlayerAction::MoveRight, GamepadButton::DPadRight)
        .with(PlayerAction::Fire, GamepadButton::South)
        .with_gamepad(gamepad)
    }
}
