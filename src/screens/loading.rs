use crate::{assets_load::ResourceHandles, ui::font::FontAsset};
use bevy::prelude::*;

use crate::screens::Screen;
use crate::ui::widget;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), load_ui)
        .add_systems(
            Update,
            (check_assets, animate_loading_blink).run_if(in_state(Screen::Loading)),
        );
}

#[derive(Component, Clone)]
struct LoadingBlink {
    min_alpha: f32,
    max_alpha: f32,
    timer: Timer,
}

/// 加载页面，“资源加载中...”文字闪烁效果
fn load_ui(mut commands: Commands, font_asset: Option<Res<FontAsset>>) {
    let mut entity = commands.spawn((
        DespawnOnExit(Screen::Loading),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        },
    ));

    if let Some(font) = font_asset {
        entity.with_children(|parent| {
            parent.spawn((
                widget::label("Loading...", 18.0, &font),
                LoadingBlink {
                    min_alpha: 0.25,
                    max_alpha: 1.0,
                    timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                },
            ));
        });
    }
}

fn check_assets(resource_handles: Res<ResourceHandles>, mut next_state: ResMut<NextState<Screen>>) {
    if resource_handles.is_all_done() {
        next_state.set(Screen::Menu);
    } else {
        info!("资源加载中...");
    }
}

fn animate_loading_blink(time: Res<Time>, mut q: Query<(&mut LoadingBlink, &mut TextColor)>) {
    for (mut blink, mut color) in &mut q {
        blink.timer.tick(time.delta());

        // 0..1（按 timer 进度）→ 0..1（平滑往返）
        let t = blink.timer.fraction();
        let wave = (t * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        let a = (blink.min_alpha + wave * (blink.max_alpha - blink.min_alpha)).clamp(0.0, 1.0);

        color.0.set_alpha(a);
    }
}
