use crate::ui::font::FontAsset;
use bevy::prelude::*;

#[derive(Bundle, Clone)]
pub struct LabelBundle {
    pub name: Name,
    pub text: Text,
    pub font: TextFont,
    pub color: TextColor,
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
