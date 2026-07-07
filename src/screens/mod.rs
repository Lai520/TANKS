use bevy::prelude::*;

mod game_play;
mod loading;
mod menu;
mod stage_completion;

pub use game_play::{game_is_active, GameState};

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Screen {
    #[default]
    Splash,
    Loading,
    Menu,     // 菜单
    GamePlay, // 游戏中
    GameOver, // 游戏结束
}

#[derive(Resource)]
pub struct PlayerNumber {
    pub value: u8,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .insert_resource(PlayerNumber { value: 1 })
        .add_plugins((loading::plugin, menu::plugin, game_play::plugin, stage_completion::plugin));
}
