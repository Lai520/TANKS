use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkPlugin;

mod animation;
mod assets_load;
mod audio;
mod camera;
mod collision;
mod common_component;
mod config;
mod enemy;
mod input_manager;
mod map;
mod player;
mod props;
mod resource_manage;
mod screens;
mod ui;
mod window;

fn main() -> AppExit {
    App::new()
        .add_plugins((window::plugin, LdtkPlugin)) // 前置插件加载
        .add_plugins((
            camera::plugin,          // 相机
            assets_load::plugin,     // 资源加载
            ui::plugin,              // ui 模块
            input_manager::plugin,   // 输入管理
            resource_manage::plugin, // 资源管理
            screens::plugin,         // 场景模块
            map::plguin,             // 关卡地图
            player::plugin,          // 玩家
            animation::plugin,       // 动画系统（需在 enemy 之前，保证生成动画先完成）
            enemy::plugin,           // 敌人
            collision::plugin,       // 碰撞检测
            props::plugin,           // 道具系统
        ))
        .run()
}
