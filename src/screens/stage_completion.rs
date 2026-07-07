use bevy::prelude::*;
use leafwing_input_manager::{action_state::ActionState, input_map::InputMap};

use crate::{
    audio::sound_effect,
    common_component::BulletInfo,
    enemy::EnemyNumberState,
    input_manager::PlayerAction,
    map::{MapLevel, ReloadLevel},
    player::{EnemyKillRecords, PlayerInfo, ENEMY_TANK_TYPE_COUNT},
    resource_manage::{AudioAsset, ImgAsset},
    screens::{GameState, PlayerNumber},
    ui::font::FontAsset,
};

const UI_Z: f32 = 100.;
const TICK_INTERVAL: f32 = 0.12;
const HEADER_COLOR: Color = Color::srgb(1., 0.45, 0.2);
const TEXT_COLOR: Color = Color::WHITE;

const STAGE_TITLE_Y: f32 = 220.;
const HEADER_Y: f32 = 200.;
const ROW_START_Y: f32 = 168.;
const ROW_SPACING: f32 = 26.;
const TOTAL_Y: f32 = 78.;
const P1_PTS_X: f32 = 24.;
const P1_COUNT_X: f32 = 72.;
const TANK_ICON_X: f32 = 140.;
const P2_COUNT_X: f32 = 208.;
const P2_PTS_X: f32 = 276.;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<StageCompletionAnim>()
        .add_systems(OnEnter(GameState::GameSettlement), spawn_stage_completion_ui)
        .add_systems(OnExit(GameState::GameSettlement), cleanup_stage_completion)
        .add_systems(
            Update,
            (animate_stage_completion, stage_completion_input)
                .run_if(in_state(GameState::GameSettlement)),
        );
}

#[derive(Component)]
struct StageCompletionUi;

#[derive(Component)]
struct StageCompletionController;

#[derive(Component)]
enum StageCompletionText {
    RowPointsP1(usize),
    RowCountP1(usize),
    RowCountP2(usize),
    RowPointsP2(usize),
    TotalP1,
    TotalP2,
}

#[derive(Component)]
struct StageRowPointsP1 {
    row: usize,
}

#[derive(Component)]
struct StageRowCountP1 {
    row: usize,
}

#[derive(Component)]
struct StageRowCountP2 {
    row: usize,
}

#[derive(Component)]
struct StageRowPointsP2 {
    row: usize,
}

#[derive(Component)]
struct StageTotalP1;

#[derive(Component)]
struct StageTotalP2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AnimPhase {
    Player1,
    Player2,
}

#[derive(Resource)]
struct StageCompletionAnim {
    row: usize,
    phase: AnimPhase,
    tick_timer: Timer,
    displayed: [[u32; ENEMY_TANK_TYPE_COUNT]; 2],
    finished: bool,
}

impl Default for StageCompletionAnim {
    fn default() -> Self {
        Self {
            row: 0,
            phase: AnimPhase::Player1,
            tick_timer: Timer::from_seconds(TICK_INTERVAL, TimerMode::Repeating),
            displayed: [[0; ENEMY_TANK_TYPE_COUNT]; 2],
            finished: false,
        }
    }
}

fn spawn_stage_completion_ui(
    mut commands: Commands,
    mut anim: ResMut<StageCompletionAnim>,
    player_info_query: Query<&PlayerInfo>,
    player_number: Res<PlayerNumber>,
    map_level: Res<MapLevel>,
    img_asset: Res<ImgAsset>,
    font_asset: Res<FontAsset>,
) {
    *anim = StageCompletionAnim::default();

    let menu_controller = InputMap::new([
        (PlayerAction::MoveUp, KeyCode::KeyW),
        (PlayerAction::MoveDown, KeyCode::KeyS),
        (PlayerAction::Start, KeyCode::Enter),
    ])
    .with(PlayerAction::MoveUp, GamepadButton::DPadUp)
    .with(PlayerAction::MoveDown, GamepadButton::DPadDown)
    .with(PlayerAction::Start, GamepadButton::Start);

    let players: Vec<&PlayerInfo> = player_info_query
        .iter()
        .filter(|info| info.id <= player_number.value)
        .collect();

    commands
        .spawn((
            StageCompletionUi,
            StageCompletionController,
            menu_controller,
            DespawnOnExit(GameState::GameSettlement),
            Transform::from_xyz(140., 120., UI_Z),
            Visibility::default(),
        ))
        .with_children(|root| {
            root.spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(300., 240.)),
                    ..default()
                },
                Transform::from_xyz(0., 0., 0.),
            ));

            root.spawn(stage_text(
                format!("STAGE {}", map_level.value),
                12.,
                TEXT_COLOR,
                Vec3::new(0., STAGE_TITLE_Y - 120., 1.),
                &font_asset,
            ));

            if let Some(player) = players.iter().find(|p| p.id == 1) {
                root.spawn(stage_text(
                    "I-PLAYER".to_string(),
                    10.,
                    HEADER_COLOR,
                    Vec3::new(P1_PTS_X - 120., HEADER_Y - 120., 1.),
                    &font_asset,
                ));
                root.spawn(stage_text(
                    format!("{}", player.score),
                    10.,
                    HEADER_COLOR,
                    Vec3::new(P1_PTS_X - 120., HEADER_Y - 132., 1.),
                    &font_asset,
                ));
            }

            if player_number.value >= 2 {
                if let Some(player) = players.iter().find(|p| p.id == 2) {
                    root.spawn(stage_text(
                        "II-PLAYER".to_string(),
                        10.,
                        HEADER_COLOR,
                        Vec3::new(P2_PTS_X - 120., HEADER_Y - 120., 1.),
                        &font_asset,
                    ));
                    root.spawn(stage_text(
                        format!("{}", player.score),
                        10.,
                        HEADER_COLOR,
                        Vec3::new(P2_PTS_X - 120., HEADER_Y - 132., 1.),
                        &font_asset,
                    ));
                }
            }

            for row in 0..ENEMY_TANK_TYPE_COUNT {
                let y = ROW_START_Y - row as f32 * ROW_SPACING - 120.;
                root.spawn((
                    Sprite {
                        image: enemy_tank_icon(row, &img_asset),
                        ..default()
                    },
                    Transform::from_xyz(TANK_ICON_X - 120., y, 1.),
                ));
                root.spawn((
                    StageRowPointsP1 { row },
                    StageCompletionText::RowPointsP1(row),
                    stage_text(
                        "0 PTS".to_string(),
                        9.,
                        TEXT_COLOR,
                        Vec3::new(P1_PTS_X - 120., y, 1.),
                        &font_asset,
                    ),
                ));
                root.spawn((
                    StageRowCountP1 { row },
                    StageCompletionText::RowCountP1(row),
                    stage_text(
                        "0 ←".to_string(),
                        9.,
                        TEXT_COLOR,
                        Vec3::new(P1_COUNT_X - 120., y, 1.),
                        &font_asset,
                    ),
                ));
                if player_number.value >= 2 {
                    root.spawn((
                        StageRowCountP2 { row },
                        StageCompletionText::RowCountP2(row),
                        stage_text(
                            "→ 0".to_string(),
                            9.,
                            TEXT_COLOR,
                            Vec3::new(P2_COUNT_X - 120., y, 1.),
                            &font_asset,
                        ),
                    ));
                    root.spawn((
                        StageRowPointsP2 { row },
                        StageCompletionText::RowPointsP2(row),
                        stage_text(
                            "0 PTS".to_string(),
                            9.,
                            TEXT_COLOR,
                            Vec3::new(P2_PTS_X - 120., y, 1.),
                            &font_asset,
                        ),
                    ));
                }
            }

            root.spawn(stage_text(
                "TOTAL".to_string(),
                9.,
                TEXT_COLOR,
                Vec3::new(P1_PTS_X - 120., TOTAL_Y - 120., 1.),
                &font_asset,
            ));
            root.spawn((
                StageTotalP1,
                StageCompletionText::TotalP1,
                stage_text(
                    "0".to_string(),
                    9.,
                    TEXT_COLOR,
                    Vec3::new(P1_COUNT_X - 120., TOTAL_Y - 120., 1.),
                    &font_asset,
                ),
            ));
            if player_number.value >= 2 {
                root.spawn((
                    StageTotalP2,
                    StageCompletionText::TotalP2,
                    stage_text(
                        "0".to_string(),
                        9.,
                        TEXT_COLOR,
                        Vec3::new(P2_COUNT_X - 120., TOTAL_Y - 120., 1.),
                        &font_asset,
                    ),
                ));
            }
        });
}

fn stage_text(
    content: String,
    font_size: f32,
    color: Color,
    translation: Vec3,
    font_asset: &FontAsset,
) -> impl Bundle {
    (
        Text2d::new(content),
        TextFont {
            font: font_asset.font.clone(),
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(translation),
    )
}

fn enemy_tank_icon(row: usize, img_asset: &ImgAsset) -> Handle<Image> {
    match row {
        1 => img_asset.e2_0_0.clone(),
        2 => img_asset.e3_2_0.clone(),
        _ => img_asset.e1_0_0.clone(),
    }
}

fn player_kills(players: &[&PlayerInfo], player_id: u8) -> EnemyKillRecords {
    players
        .iter()
        .find(|player| player.id == player_id)
        .map(|player| player.enemy_kills)
        .unwrap_or_default()
}

fn cleanup_stage_completion(mut anim: ResMut<StageCompletionAnim>) {
    *anim = StageCompletionAnim::default();
}

fn animate_stage_completion(
    mut commands: Commands,
    time: Res<Time>,
    mut anim: ResMut<StageCompletionAnim>,
    player_info_query: Query<&PlayerInfo>,
    player_number: Res<PlayerNumber>,
    audio_asset: Res<AudioAsset>,
    mut text_query: Query<(&StageCompletionText, &mut Text2d)>,
) {
    if anim.finished {
        return;
    }

    anim.tick_timer.tick(time.delta());
    if !anim.tick_timer.just_finished() {
        return;
    }

    let players: Vec<&PlayerInfo> = player_info_query
        .iter()
        .filter(|info| info.id <= player_number.value)
        .collect();

    let row = anim.row;
    if row >= ENEMY_TANK_TYPE_COUNT {
        anim.finished = true;
        refresh_all_text(&anim, player_number.value, &players, &mut text_query);
        return;
    }

    match anim.phase {
        AnimPhase::Player1 => {
            let target = player_kills(&players, 1).count_for_type(row);
            if anim.displayed[0][row] < target {
                anim.displayed[0][row] += 1;
                commands.spawn(sound_effect(audio_asset.bullet_hit_1.clone()));
                refresh_all_text(&anim, player_number.value, &players, &mut text_query);
                return;
            }
            anim.phase = AnimPhase::Player2;
        }
        AnimPhase::Player2 => {
            if player_number.value < 2 {
                anim.row += 1;
                anim.phase = AnimPhase::Player1;
                return;
            }
            let target = player_kills(&players, 2).count_for_type(row);
            if anim.displayed[1][row] < target {
                anim.displayed[1][row] += 1;
                commands.spawn(sound_effect(audio_asset.bullet_hit_1.clone()));
                refresh_all_text(&anim, player_number.value, &players, &mut text_query);
                return;
            }
            anim.row += 1;
            anim.phase = AnimPhase::Player1;
        }
    }
}

fn refresh_all_text(
    anim: &StageCompletionAnim,
    player_count: u8,
    players: &[&PlayerInfo],
    text_query: &mut Query<(&StageCompletionText, &mut Text2d)>,
) {
    let points_per_kill_p1 = [
        player_kills(players, 1)
            .points_per_kill(0)
            .max(default_points_for_row(0)),
        player_kills(players, 1)
            .points_per_kill(1)
            .max(default_points_for_row(1)),
        player_kills(players, 1)
            .points_per_kill(2)
            .max(default_points_for_row(2)),
    ];
    let points_per_kill_p2 = [
        player_kills(players, 2)
            .points_per_kill(0)
            .max(default_points_for_row(0)),
        player_kills(players, 2)
            .points_per_kill(1)
            .max(default_points_for_row(1)),
        player_kills(players, 2)
            .points_per_kill(2)
            .max(default_points_for_row(2)),
    ];

    for (label, mut text) in text_query.iter_mut() {
        match label {
            StageCompletionText::RowPointsP1(row) => {
                let points = anim.displayed[0][*row] as usize * points_per_kill_p1[*row];
                **text = format!("{points} PTS");
            }
            StageCompletionText::RowCountP1(row) => {
                **text = format!("{} ←", anim.displayed[0][*row]);
            }
            StageCompletionText::RowCountP2(row) if player_count >= 2 => {
                **text = format!("→ {}", anim.displayed[1][*row]);
            }
            StageCompletionText::RowPointsP2(row) if player_count >= 2 => {
                let points = anim.displayed[1][*row] as usize * points_per_kill_p2[*row];
                **text = format!("{points} PTS");
            }
            StageCompletionText::TotalP1 => {
                **text = anim.displayed[0].iter().sum::<u32>().to_string();
            }
            StageCompletionText::TotalP2 if player_count >= 2 => {
                **text = anim.displayed[1].iter().sum::<u32>().to_string();
            }
            _ => {}
        }
    }
}

fn default_points_for_row(row: usize) -> usize {
    match row {
        1 => 200,
        2 => 500,
        _ => 100,
    }
}

fn stage_completion_input(
    mut commands: Commands,
    anim: Res<StageCompletionAnim>,
    mut map_level: ResMut<MapLevel>,
    mut enemy_number_state: ResMut<EnemyNumberState>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut reload_level: MessageWriter<ReloadLevel>,
    mut player_info_query: Query<&mut PlayerInfo>,
    controller_query: Query<&ActionState<PlayerAction>, With<StageCompletionController>>,
    audio_asset: Res<AudioAsset>,
    bullet_query: Query<Entity, With<BulletInfo>>,
) {
    if !anim.finished {
        return;
    }

    for action_state in &controller_query {
        if action_state.just_pressed(&PlayerAction::Start) {
            map_level.value += 1;
            enemy_number_state.reset();
            for mut player in &mut player_info_query {
                player.enemy_kills.reset();
            }
            for bullet in &bullet_query {
                commands.entity(bullet).despawn();
            }
            reload_level.write(ReloadLevel);
            next_game_state.set(GameState::None);
            commands.spawn(sound_effect(audio_asset.start_menu.clone()));
            break;
        }
    }
}
