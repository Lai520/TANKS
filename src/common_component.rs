use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{collision::collision_groups, resource_manage::ImgAsset};

/// 朝向
#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

/// 坦克旋转角度
pub const ROTATION: [f32; 4] = [
    0.0,
    std::f32::consts::PI,
    std::f32::consts::FRAC_PI_2,
    -std::f32::consts::FRAC_PI_2,
];

/// 动画组件
#[derive(Component)]
pub struct MoveAnimation {
    pub timer: Timer,               // 帧持续时间
    pub frames: Vec<Handle<Image>>, // 动画精灵
    pub cur_frames: usize,          // 当前帧
    pub playing: bool,              // 是否播放
}

/// 生成坦克时坦克组件
#[derive(Component)]
pub struct TankId(pub u8);

/// 玩家移动循环动画组件
pub fn player_loop_animation(player_id: u8, level: u8, img_asset: &Res<ImgAsset>) -> MoveAnimation {
    let frames = match player_id {
        1 => match level {
            1 => [img_asset.p1_1_0.clone(), img_asset.p1_1_1.clone()],
            2 => [img_asset.p1_2_0.clone(), img_asset.p1_2_1.clone()],
            3 => [img_asset.p1_3_0.clone(), img_asset.p1_3_1.clone()],
            _ => [img_asset.p1_0_0.clone(), img_asset.p1_0_1.clone()],
        },
        2 => match level {
            1 => [img_asset.p2_1_0.clone(), img_asset.p2_1_1.clone()],
            2 => [img_asset.p2_2_0.clone(), img_asset.p2_2_1.clone()],
            3 => [img_asset.p2_3_0.clone(), img_asset.p2_3_1.clone()],
            _ => [img_asset.p2_0_0.clone(), img_asset.p2_0_1.clone()],
        },
        _ => [img_asset.p1_0_0.clone(), img_asset.p1_0_1.clone()],
    }
    .to_vec();

    MoveAnimation {
        timer: Timer::from_seconds(0.06, TimerMode::Repeating),
        frames,
        cur_frames: 0,
        playing: false,
    }
}

/// 敌人移动循环动画组件
pub fn enemy_loop_animation(
    enemy_type: u8,
    is_carrying_props: bool,
    img_asset: &Res<ImgAsset>,
) -> MoveAnimation {
    let frames = match is_carrying_props {
        false => match enemy_type {
            2 => [img_asset.e2_0_0.clone(), img_asset.e2_0_1.clone()],
            3 => [img_asset.e3_2_0.clone(), img_asset.e3_2_1.clone()],
            _ => [img_asset.e1_0_0.clone(), img_asset.e1_0_1.clone()],
        },
        true => match enemy_type {
            2 => [img_asset.e2_1_0.clone(), img_asset.e2_1_1.clone()],
            3 => [img_asset.e3_3_0.clone(), img_asset.e3_3_1.clone()],
            _ => [img_asset.e1_1_0.clone(), img_asset.e1_1_1.clone()],
        },
    }
    .to_vec();

    MoveAnimation {
        timer: Timer::from_seconds(0.06, TimerMode::Repeating),
        frames,
        cur_frames: 0,
        playing: false,
    }
}

/// 坦克生成动画
pub fn spawn_animation(
    img_asset: &Res<ImgAsset>,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    transform: Transform,
    id: u8,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(15), 4, 1, None, None);
    let atlas_layout = atlas_layouts.add(layout);
    (
        transform,
        TankId(id),
        RigidBody::Static,
        Collider::rectangle(16., 16.),
        Friction::new(0.),
        Restitution::ZERO,
        CollisionEventsEnabled,
        collision_groups::enemy(),
        Sprite::from_atlas_image(
            img_asset.spawn.clone(),
            TextureAtlas {
                layout: atlas_layout,
                index: 0,
            },
        ),
        SpawnAnimation {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            first: 0,
            last: 3,
            transform: transform,
            loop_num: 2,
        },
    )
}

/// 敌坦克生成消息
#[derive(Message)]
pub struct EnemyTankSpawn {
    /// 生成动画实体（就地转换为敌坦克，避免 despawn 后 spawn 造成实体替换）
    pub entity: Entity,
    pub transform: Transform,
}

/// 玩家生成消息
#[derive(Message)]
pub struct PlayerTankSpawn {
    pub transform: Transform,
    pub id: u8, // 生成坦克的ID 1=p1，2=p2
}

/// 道具生成消息
#[derive(Message)]
pub struct PropsSpawn;

/// 炮弹组件
#[derive(Component, Debug)]
pub struct BulletInfo {
    pub entity: Entity, // 发射炮弹的玩家实体
    pub level: u8,      // 发射炮弹的玩家等级 1-3 3级可以摧毁石墙
    pub horde: u8, // 阵营 1=玩家 2=敌人 玩家命中玩家，会导致玩家闪烁僵直。玩家命中敌人，将摧毁敌人。敌人命中玩家，将摧毁玩家
}

/// 爆炸效果动画组件
#[derive(Component)]
pub struct EffectAnimation {
    pub timer: Timer, // 帧持续时间
    pub first: usize, // 第一帧
    pub last: usize,  // 最后一帧
}

/// 坦克生成动画组件
#[derive(Component)]
pub struct SpawnAnimation {
    pub timer: Timer,         // 帧持续时间
    pub first: usize,         // 第一帧
    pub last: usize,          // 最后一帧
    pub loop_num: u8,         // 循环次数
    pub transform: Transform, // 生成位置
}

/// 防水层
#[derive(Component)]
pub struct Waterproofer;

/// 添加防水层
pub fn add_waterproofer(img_asset: &Res<ImgAsset>) -> impl Bundle {
    (
        Waterproofer,
        Sprite::from_image(img_asset.river_shield.clone()),
    )
}

/// 防弹层
#[derive(Component)]
pub struct BulletproofLayer {
    pub timer: Timer,        // 防弹层持续时间
    pub frames_timer: Timer, // 帧持续时间
    pub first: usize,        // 第一帧
    pub last: usize,         // 最后一帧
    pub cur_frames: usize,   // 当前帧
}

/// 添加防护罩动画
pub fn add_bulletproof(
    img_asset: &Res<ImgAsset>,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    duration: f32,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 2, 1, None, None);
    let atlas_layout = atlas_layouts.add(layout);

    (
        BulletproofLayer {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            frames_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            first: 0,
            last: 1,
            cur_frames: 0,
        },
        Sprite::from_atlas_image(
            img_asset.born.clone(),
            TextureAtlas {
                layout: atlas_layout,
                index: 0,
            },
        ),
        Transform::from_xyz(0., 0., 2.),
    )
}
