use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, PartialEq, Eq, Hash, Debug, Reflect)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Move,
    MoveUp,    // 向上移动
    MoveDown,  // 向下移动
    MoveLeft,  // 向左移动
    MoveRight, // 向右移动
    Fire,      // 开火
    Pause,     // 暂停
    Start,     // 开始
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
}
