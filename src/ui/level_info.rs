use bevy::prelude::*;

use crate::{
    enemy::EnemyNumberState,
    map::MapLevel,
    player::PlayerInfo,
    resource_manage::ImgAsset,
    screens::{PlayerNumber, Screen},
    ui::font::FontAsset,
};

const REMAINING_ENEMIES_X: f32 = 250.;
const REMAINING_ENEMIES_Y: f32 = 230.;
const ENEMY_ICON_SPACING: f32 = 10.;
const PLAYER_CHANCE_X: f32 = 250.;
const PLAYER_CHANCE_BOTTOM_Y: f32 = 18.;
const FLAG_SPRITE_HEIGHT: f32 = 15.;
const FLAG_OFFSET_X: f32 = 10.;
const FLAG_PLAYER_GAP: f32 = 16.;
const PLAYER_ICON_HALF_HEIGHT: f32 = 4.;
const PLAYER_CHANCE_TEXT_HALF_HEIGHT: f32 = 5.;
/// 1P 机会数底部与 2P p_icon 顶部之间的间距
const PLAYER_ROW_GAP: f32 = 12.;
const PLAYER_LABEL_OFFSET_X: f32 = 0.;
const PLAYER_ICON_OFFSET_X: f32 = 14.;
const PLAYER_CHANCE_VALUE_OFFSET_X: f32 = 14.;
const PLAYER_CHANCE_VALUE_OFFSET_Y: f32 = -9.;
const FLAG_LEVEL_OFFSET_X: f32 = 12.;
const PLAYER_LABEL_FONT_SIZE: f32 = 10.;
const LEVEL_NUMBER_FONT_SIZE: f32 = 10.;
const UI_Z: f32 = 10.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GamePlay), render_level_info)
        .add_systems(
            Update,
            (
                setup_player_chance_info,
                update_remaining_enemies,
                update_player_chance_info,
                update_level_number,
            )
                .chain()
                .run_if(in_state(Screen::GamePlay)),
        );
}

/// 剩余敌人数量展示容器
#[derive(Component)]
struct RemainingEnemies;

/// 玩家机会信息容器
#[derive(Component)]
struct PlayerChanceInfo;

/// 单个玩家的机会行
#[derive(Component)]
struct PlayerChanceRow {
    player_id: u8,
}

/// 玩家阵营旗帜
#[derive(Component)]
struct PlayerCampFlag;

/// 关卡数字
#[derive(Component)]
struct LevelNumberLabel;

/// 玩家标识文本（1P / 2P）
#[derive(Component)]
struct PlayerChanceLabel;

/// 玩家机会图标
#[derive(Component)]
struct PlayerChanceIcon;

/// 玩家剩余机会数字
#[derive(Component)]
struct PlayerChanceValue;

fn player_label(player_id: u8) -> String {
    format!("{player_id}P")
}

fn player_row_stride() -> f32 {
    PLAYER_ICON_HALF_HEIGHT
        + PLAYER_ROW_GAP
        + (-PLAYER_CHANCE_VALUE_OFFSET_Y + PLAYER_CHANCE_TEXT_HALF_HEIGHT)
}

fn flag_row_y(player_count: usize) -> f32 {
    if player_count == 0 {
        return FLAG_SPRITE_HEIGHT / 2.;
    }
    let top_player_anchor = (player_count - 1) as f32 * player_row_stride();
    top_player_anchor + PLAYER_ICON_HALF_HEIGHT + FLAG_PLAYER_GAP + FLAG_SPRITE_HEIGHT / 2.
}

/// 左侧关卡敌人信息
fn render_level_info(
    mut commands: Commands,
    img_asset: Res<ImgAsset>,
    enemy_number_state: Res<EnemyNumberState>,
) {
    commands
        .spawn((
            RemainingEnemies,
            Transform::from_xyz(REMAINING_ENEMIES_X, REMAINING_ENEMIES_Y, UI_Z),
            Visibility::default(),
        ))
        .with_children(|parent| {
            for i in 0..enemy_number_state.max_enemy_count {
                let col = (i % 2) as f32;
                let row = (i / 2) as f32;
                parent.spawn((
                    Sprite {
                        image: img_asset.e_icon.clone(),
                        ..default()
                    },
                    Transform::from_xyz(col * ENEMY_ICON_SPACING, row * -ENEMY_ICON_SPACING, 1.),
                ));
            }
        });
}

/// 根据剩余敌人数量，从底部向上移除多余的 e_icon 精灵
fn update_remaining_enemies(
    mut commands: Commands,
    enemy_number_state: Res<EnemyNumberState>,
    remaining_query: Query<&Children, With<RemainingEnemies>>,
) {
    let remain_enemy_count =
        enemy_number_state.max_enemy_count - enemy_number_state.spawned_enemy_count;

    if let Ok(children) = remaining_query.single() {
        let current_count = children.len();
        if current_count > remain_enemy_count {
            for child in children.iter().skip(remain_enemy_count) {
                commands.entity(child).despawn();
            }
        }
    }
}

/// 添加玩家机会信息（位于屏幕左下方）
fn setup_player_chance_info(
    mut commands: Commands,
    player_info_query: Query<&PlayerInfo>,
    existing: Query<(), With<PlayerChanceInfo>>,
    player_number: Res<PlayerNumber>,
    map_level: Res<MapLevel>,
    img_asset: Res<ImgAsset>,
    font_asset: Res<FontAsset>,
) {
    if !existing.is_empty() {
        return;
    }

    let mut players: Vec<&PlayerInfo> = player_info_query
        .iter()
        .filter(|info| info.id <= player_number.value)
        .collect();
    if players.is_empty() {
        return;
    }
    players.sort_by_key(|info| info.id);

    commands
        .spawn((
            PlayerChanceInfo,
            Transform::from_xyz(PLAYER_CHANCE_X, PLAYER_CHANCE_BOTTOM_Y, UI_Z),
            Visibility::default(),
        ))
        .with_children(|parent| {
            let mut row_y = 0.;

            for player_info in players.iter().rev() {
                parent
                    .spawn((
                        Visibility::default(),
                        PlayerChanceRow {
                            player_id: player_info.id,
                        },
                        Transform::from_xyz(0., row_y, 1.),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            PlayerChanceLabel,
                            Text2d::new(player_label(player_info.id)),
                            TextFont {
                                font: font_asset.font.clone(),
                                font_size: PLAYER_LABEL_FONT_SIZE,
                                ..default()
                            },
                            TextColor::WHITE,
                            Transform::from_xyz(PLAYER_LABEL_OFFSET_X, 0., 1.),
                        ));
                        row.spawn((
                            PlayerChanceIcon,
                            Sprite {
                                image: img_asset.p_icon.clone(),
                                ..default()
                            },
                            Transform::from_xyz(PLAYER_ICON_OFFSET_X, 0., 1.),
                        ));
                        row.spawn((
                            PlayerChanceValue,
                            Text2d::new(format!("{}", player_info.chance)),
                            TextFont {
                                font: font_asset.font.clone(),
                                font_size: PLAYER_LABEL_FONT_SIZE,
                                ..default()
                            },
                            TextColor::WHITE,
                            Transform::from_xyz(
                                PLAYER_CHANCE_VALUE_OFFSET_X,
                                PLAYER_CHANCE_VALUE_OFFSET_Y,
                                1.,
                            ),
                        ));
                    });
                row_y += player_row_stride();
            }

            parent
                .spawn((
                    Visibility::default(),
                    Transform::from_xyz(FLAG_OFFSET_X, flag_row_y(players.len()), 1.),
                ))
                .with_children(|flag_row| {
                    flag_row.spawn((
                        PlayerCampFlag,
                        Sprite {
                            image: img_asset.flag.clone(),
                            ..default()
                        },
                        Transform::from_xyz(0., 0., 1.),
                    ));
                    flag_row.spawn((
                        LevelNumberLabel,
                        Text2d::new(format!("{}", map_level.value)),
                        TextFont {
                            font: font_asset.font.clone(),
                            font_size: LEVEL_NUMBER_FONT_SIZE,
                            ..default()
                        },
                        TextColor::WHITE,
                        Transform::from_xyz(FLAG_LEVEL_OFFSET_X, 0., 1.),
                    ));
                });
        });
}

/// 根据 PlayerInfo 更新各玩家剩余机会数字
fn update_player_chance_info(
    player_info_query: Query<&PlayerInfo>,
    row_query: Query<(&PlayerChanceRow, &Children)>,
    mut chance_value_query: Query<&mut Text2d, With<PlayerChanceValue>>,
) {
    for player_info in &player_info_query {
        for (row, children) in &row_query {
            if row.player_id != player_info.id {
                continue;
            }

            for child in children.iter() {
                if let Ok(mut text) = chance_value_query.get_mut(child) {
                    **text = format!("{}", player_info.chance);
                    break;
                }
            }
        }
    }
}

/// 更新 flag 右侧的关卡数字
fn update_level_number(
    map_level: Res<MapLevel>,
    mut level_query: Query<&mut Text2d, With<LevelNumberLabel>>,
) {
    if !map_level.is_changed() {
        return;
    }

    for mut text in &mut level_query {
        **text = format!("{}", map_level.value);
    }
}
