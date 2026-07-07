use bevy::prelude::*;

use crate::{
    assets_load::{LoadResource, ResourceHandles},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<FontAsset>()
        .load_resource::<FontAsset>()
        .add_systems(Update, font_loaded.run_if(in_state(Screen::Splash)));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct FontAsset {
    #[dependency]
    pub font: Handle<Font>,
}

/// 加载字体
impl FromWorld for FontAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            font: assets.load("font/SIMHEI.TTF"),
        }
    }
}

/// 字体加载完毕
fn font_loaded(resource_handles: Res<ResourceHandles>, mut next_state: ResMut<NextState<Screen>>) {
    if resource_handles.is_all_done() {
        next_state.set(Screen::Loading);
    }
}
