use bevy::prelude::*;

use crate::{
    collision::add_prop_collision, common_component::PropsSpawn, map::MapBounds, props::PropType,
    resource_manage::ImgAsset, screens::{game_is_active, Screen},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_prop
            .run_if(in_state(Screen::GamePlay).and(game_is_active))
            .run_if(resource_exists::<MapBounds>),
    );
}

#[derive(Component)]
pub struct Prop(pub PropType);

/// 接收道具生成消息生成道具
fn spawn_prop(
    mut commands: Commands,
    img_asset: Res<ImgAsset>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut messages: MessageReader<PropsSpawn>,
    prop_query: Query<Entity, With<Prop>>,
    map_bounds: Res<MapBounds>,
) {
    for _ in messages.read() {
        for prop_entity in prop_query.iter() {
            commands.entity(prop_entity).despawn();
        }
        let prop_type = PropType::random();
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        let position = map_bounds.random_grid_center();
        commands.spawn((
            Prop(prop_type),
            Sprite::from_atlas_image(
                img_asset.prop.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: prop_type as usize,
                },
            ),
            Transform::from_translation(position),
            add_prop_collision(),
        ));
    }
}
