use crate::assets_load::LoadResource;
use bevy::prelude::*;
use bevy_ecs_ldtk::assets::LdtkProject;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ImgAsset>()
        .load_resource::<ImgAsset>()
        .register_type::<AudioAsset>()
        .load_resource::<AudioAsset>();
}

/// 地图图片资源
#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ImgAsset {
    #[dependency]
    pub map: Handle<LdtkProject>, // 关卡地图文件
    #[dependency]
    pub start_bg: Handle<Image>, // 开始页面背景
    #[dependency]
    pub camp_break: Handle<Image>, // 堡垒废墟
    #[dependency]
    pub boom: Handle<Image>, // 爆炸
    #[dependency]
    pub born: Handle<Image>, // 防护罩
    #[dependency]
    pub bullet: Handle<Image>, // 炮弹
    #[dependency]
    pub prop: Handle<Image>, // 道具
    #[dependency]
    pub river_shield: Handle<Image>, // 防水罩
    #[dependency]
    pub spawn: Handle<Image>, // 孵化星星
    #[dependency]
    pub p1_0_0: Handle<Image>, // 玩家1_等级0
    #[dependency]
    pub p1_0_1: Handle<Image>, // 玩家1_等级0
    #[dependency]
    pub p1_1_0: Handle<Image>, // 玩家1_等级1
    #[dependency]
    pub p1_1_1: Handle<Image>, // 玩家1_等级1
    #[dependency]
    pub p1_2_0: Handle<Image>, // 玩家1_等级2
    #[dependency]
    pub p1_2_1: Handle<Image>, // 玩家1_等级2
    #[dependency]
    pub p1_3_0: Handle<Image>, // 玩家1_等级3
    #[dependency]
    pub p1_3_1: Handle<Image>, // 玩家1_等级3
    #[dependency]
    pub p2_0_0: Handle<Image>, // 玩家2_等级0
    #[dependency]
    pub p2_0_1: Handle<Image>, // 玩家2_等级0
    #[dependency]
    pub p2_1_0: Handle<Image>, // 玩家2_等级1
    #[dependency]
    pub p2_1_1: Handle<Image>, // 玩家2_等级1
    #[dependency]
    pub p2_2_0: Handle<Image>, // 玩家2_等级2
    #[dependency]
    pub p2_2_1: Handle<Image>, // 玩家2_等级2
    #[dependency]
    pub p2_3_0: Handle<Image>, // 玩家2_等级3
    #[dependency]
    pub p2_3_1: Handle<Image>, // 玩家2_等级3
    #[dependency]
    pub e1_0_0: Handle<Image>, // 敌人类型1
    #[dependency]
    pub e1_0_1: Handle<Image>, // 敌人类型1
    #[dependency]
    pub e1_1_0: Handle<Image>, // 敌人类型--携带道具
    #[dependency]
    pub e1_1_1: Handle<Image>, // 敌人类型--携带道具
    #[dependency]
    pub e2_0_0: Handle<Image>, // 敌人类型2
    #[dependency]
    pub e2_0_1: Handle<Image>, // 敌人类型2
    #[dependency]
    pub e2_1_0: Handle<Image>, // 敌人类型2--携带道具
    #[dependency]
    pub e2_1_1: Handle<Image>, // 敌人类型2--携带道具
    #[dependency]
    pub e3_0_0: Handle<Image>, // 敌人类型3--生命值1
    #[dependency]
    pub e3_0_1: Handle<Image>, // 敌人类型3--生命值1
    #[dependency]
    pub e3_1_0: Handle<Image>, // 敌人类型3--生命值2
    #[dependency]
    pub e3_1_1: Handle<Image>, // 敌人类型3--生命值2
    #[dependency]
    pub e3_2_0: Handle<Image>, // 敌人类型3--生命值3
    #[dependency]
    pub e3_2_1: Handle<Image>, // 敌人类型3--生命值3
    #[dependency]
    pub e3_3_0: Handle<Image>, // 敌人类型3--携带道具
    #[dependency]
    pub e3_3_1: Handle<Image>, // 敌人类型3--携带道具
    #[dependency]
    pub e_icon: Handle<Image>, // 敌人icon
    #[dependency]
    pub p_icon: Handle<Image>, // 玩家icon
    #[dependency]
    pub flag: Handle<Image>, // 旗帜
    #[dependency]
    pub gameover: Handle<Image>, // 游戏结束
    #[dependency]
    pub big_gameover: Handle<Image>, // 游戏结束_大
}

impl FromWorld for ImgAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            start_bg: assets.load("ui/start-bg.png"),
            map: assets.load("map/map.ldtk"),
            camp_break: assets.load("map/camp_break.png"),
            boom: assets.load("prop/boom.png"),
            born: assets.load("prop/born.png"),
            bullet: assets.load("prop/bullet.png"),
            prop: assets.load("prop/prop.png"),
            river_shield: assets.load("prop/river_shield.png"),
            spawn: assets.load("prop/spawn.png"),
            p1_0_0: assets.load("player/p1/p1_0_0.png"),
            p1_0_1: assets.load("player/p1/p1_0_1.png"),
            p1_1_0: assets.load("player/p1/p1_1_0.png"),
            p1_1_1: assets.load("player/p1/p1_1_1.png"),
            p1_2_0: assets.load("player/p1/p1_2_0.png"),
            p1_2_1: assets.load("player/p1/p1_2_1.png"),
            p1_3_0: assets.load("player/p1/p1_3_0.png"),
            p1_3_1: assets.load("player/p1/p1_3_1.png"),
            p2_0_0: assets.load("player/p2/p2_0_0.png"),
            p2_0_1: assets.load("player/p2/p2_0_1.png"),
            p2_1_0: assets.load("player/p2/p2_1_0.png"),
            p2_1_1: assets.load("player/p2/p2_1_1.png"),
            p2_2_0: assets.load("player/p2/p2_2_0.png"),
            p2_2_1: assets.load("player/p2/p2_2_1.png"),
            p2_3_0: assets.load("player/p2/p2_3_0.png"),
            p2_3_1: assets.load("player/p2/p2_3_1.png"),
            e1_0_0: assets.load("badTank/e1_0_0.png"),
            e1_0_1: assets.load("badTank/e1_0_1.png"),
            e1_1_0: assets.load("badTank/e1_1_0.png"),
            e1_1_1: assets.load("badTank/e1_1_1.png"),
            e2_0_0: assets.load("badTank/e2_0_0.png"),
            e2_0_1: assets.load("badTank/e2_0_1.png"),
            e2_1_0: assets.load("badTank/e2_1_0.png"),
            e2_1_1: assets.load("badTank/e2_1_1.png"),
            e3_0_0: assets.load("badTank/e3_0_0.png"),
            e3_0_1: assets.load("badTank/e3_0_1.png"),
            e3_1_0: assets.load("badTank/e3_1_0.png"),
            e3_1_1: assets.load("badTank/e3_1_1.png"),
            e3_2_0: assets.load("badTank/e3_2_0.png"),
            e3_2_1: assets.load("badTank/e3_2_1.png"),
            e3_3_0: assets.load("badTank/e3_3_0.png"),
            e3_3_1: assets.load("badTank/e3_3_1.png"),
            e_icon: assets.load("ui/eIcon.png"),
            p_icon: assets.load("ui/pIcon.png"),
            flag: assets.load("ui/flag.png"),
            gameover: assets.load("ui/gameover.png"),
            big_gameover: assets.load("ui/gameoverBig.png"),
        }
    }
}

/// 音频资源
#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AudioAsset {
    #[dependency]
    pub score1000: Handle<AudioSource>, // 1000分音效
    #[dependency]
    pub add_life: Handle<AudioSource>, // 添加生命音效
    #[dependency]
    pub big_explosion: Handle<AudioSource>, // 大爆炸 玩家和堡垒被摧毁音效
    #[dependency]
    pub bullet_explosion: Handle<AudioSource>, // 爆炸 子弹摧毁敌人音效
    #[dependency]
    pub bullet_hit_1: Handle<AudioSource>, // 子弹命中敌人、钢铁墙、边界音效
    #[dependency]
    pub bullet_hit_2: Handle<AudioSource>, // 子弹命中玩家
    #[dependency]
    pub game_over: Handle<AudioSource>, // 游戏结束
    #[dependency]
    pub game_pause: Handle<AudioSource>, // 游戏暂停
    #[dependency]
    pub mode_switch: Handle<AudioSource>, // 菜单切换
    #[dependency]
    pub player_move: Handle<AudioSource>, // 移动
    #[dependency]
    pub player_fire: Handle<AudioSource>, // 玩家开火
    #[dependency]
    pub powerup_appear: Handle<AudioSource>, // 摧毁敌人道具出现
    #[dependency]
    pub powerup_pick: Handle<AudioSource>, // 拾取道具
    #[dependency]
    pub prop_show: Handle<AudioSource>, // 掉落道具音效
    #[dependency]
    pub start_menu: Handle<AudioSource>, // 开始bgm
}

impl FromWorld for AudioAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            score1000: assets.load("audio/1000.ogg"),
            add_life: assets.load("audio/add_life.ogg"),
            big_explosion: assets.load("audio/big_explosion.ogg"),
            bullet_explosion: assets.load("audio/bullet_explosion.ogg"),
            bullet_hit_1: assets.load("audio/bullet_hit_1.ogg"),
            bullet_hit_2: assets.load("audio/bullet_hit_2.ogg"),
            game_over: assets.load("audio/game_over.ogg"),
            game_pause: assets.load("audio/game_pause.ogg"),
            mode_switch: assets.load("audio/mode_switch.ogg"),
            player_move: assets.load("audio/move.ogg"),
            player_fire: assets.load("audio/player_fire.ogg"),
            powerup_appear: assets.load("audio/powerup_appear.ogg"),
            powerup_pick: assets.load("audio/powerup_pick.ogg"),
            prop_show: assets.load("audio/prop_show.ogg"),
            start_menu: assets.load("audio/start_menu.ogg"),
        }
    }
}
