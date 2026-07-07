use bevy::prelude::*;

/// bgm 播放
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Music;

/// 循环播放一段bgm
pub fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::LOOP, Music)
}

/// 音效
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SoundEffect;

/// 播放音效
pub fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::DESPAWN, SoundEffect)
}
