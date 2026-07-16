/// 每关最大敌对坦克数量
pub const MAX_ENEMY_TANK_COUNT: usize = 20;

/// 关卡一次最多存在敌对坦克数量
pub const MAX_ENEMY_TANK_COUNT_PER_STAGE: usize = 6;

/// 地图格子边长
pub const TILE_SIZE: f32 = 16.0;

/// 坦克渲染层级（需高于冰面/河流等地图层）
pub const TANK_RENDER_Z: f32 = 4.0;

/// 玩家移动速度
pub const PLAYER_MOVE_SPEED: f32 = 50.0;

/// 冰面加速（每秒向目标速度靠近的量）
pub const ICE_ACCELERATION: f32 = 80.0;

/// 冰面摩擦减速（每秒无输入时速度衰减的量）
pub const ICE_FRICTION: f32 = 35.0;

/// 冰面最小有效速度，低于此值视为停止
pub const ICE_MIN_SPEED: f32 = 0.5;

/// 敌对轻型坦克移动速度
pub const ENEMY_LIGHT_TANK_MOVE_SPEED: f32 = 40.;

/// 敌对快速坦克移动速度
pub const FAST_TANK_MOVE_SPEED: f32 = ENEMY_LIGHT_TANK_MOVE_SPEED * 2.;

/// 敌对重型坦克移动速度
pub const ENEMY_HEAVY_TANK_MOVE_SPEED: f32 = ENEMY_LIGHT_TANK_MOVE_SPEED * 1.1;

/// 敌对坦克生成间隔（出生点未被占用时生效）---秒
pub const ENEMY_TANK_GENERATE_INTERVAL: f32 = 3.0;

/// 敌对坦克转向间隔---秒
pub const ENEMY_TANK_TURN_INTERVAL: f32 = 1.5;

/// 敌对坦克开火间隔---秒
pub const ENEMY_TANK_FIRE_INTERVAL: f32 = 1.0;

/// 敌对坦克冷却时长随机倍率下限
pub const ENEMY_ACTION_INTERVAL_MIN: f32 = 0.5;
/// 敌对坦克冷却时长随机倍率上限
pub const ENEMY_ACTION_INTERVAL_MAX: f32 = 1.5;
/// 敌对坦克转向冷却结束后实际转向的概率
pub const ENEMY_TURN_CHANCE: f64 = 0.65;
/// 敌对坦克开火冷却结束后实际开火的概率
pub const ENEMY_FIRE_CHANCE: f64 = 0.55;

/// 出生时无敌时间
pub const NO_INVINCIBLE_TIME: f32 = 6.0;

/// 防护罩无敌时间
pub const NO_INVINCIBLE_TIME_OF_PROTECT: f32 = 15.0;

/// 拾取定时器持续时间
pub const PICK_UP_TIME: f32 = 10.0;

/// 工兵铲持续时间
pub const SHOVEL_EFFECT_TIME: f32 = 10.0;

/// 玩家子弹射速倍率----0级
pub const PLAYER_BULLET_SPEED: f32 = 1.0;

/// 玩家子弹射速倍率----1级
pub const PLAYER_BULLET_SPEED_1: f32 = 1.2;

/// 玩家子弹射速倍率----2级
pub const PLAYER_BULLET_SPEED_2: f32 = 1.0;

/// 玩家子弹射速倍率----3级
pub const PLAYER_BULLET_SPEED_3: f32 = 1.2;

/// 玩家被玩家击中呆滞时长
pub const PLAYER_IDLE: f32 = 2.0;

/// 玩家机会
pub const PLAYER_CHANCE: u8 = 2;
