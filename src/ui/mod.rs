use bevy::prelude::*;

pub mod font;
mod level_info;
pub mod widget;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((font::plugin, level_info::plugin));
}
