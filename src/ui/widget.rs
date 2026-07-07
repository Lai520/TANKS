use crate::ui::font::FontAsset;
use bevy::prelude::*;
use std::borrow::Cow;

#[derive(Bundle, Clone)]
pub struct LabelBundle {
    pub name: Name,
    pub text: Text,
    pub font: TextFont,
    pub color: TextColor,
}

/// 一个根级UI节点，可填充整个窗口并居中显示其内容。
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        // 不要阻止其他 UI 根的拾取事件。
        Pickable::IGNORE,
    )
}

/// 文本标签
pub fn label(text: impl Into<String>, size: f32, font_asset: &FontAsset) -> LabelBundle {
    LabelBundle {
        name: Name::new("Label"),
        text: Text::new(text),
        font: TextFont {
            font: font_asset.font.clone(),
            font_size: size,
            ..default()
        },
        color: TextColor::from(Color::WHITE),
    }
}
