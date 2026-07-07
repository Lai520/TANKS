use avian2d::{math::AdjustPrecision, prelude::*};
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{common_component::BulletInfo, screens::{game_is_active, Screen}};

mod collision_event;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        PhysicsPlugins::default().with_collision_hooks::<BulletCollisionHooks>(),
        // PhysicsDebugPlugin,
    ))
    .insert_resource(Gravity::ZERO)
    .add_plugins(collision_event::plugin)
    .add_systems(
        FixedUpdate,
        tank_move_and_slide.run_if(in_state(Screen::GamePlay).and(game_is_active)),
    );
}

/// 爆炸消息
#[derive(Message)]
pub struct BoomMsg {
    pub transform: Transform, // 爆炸位置
    pub boom_level: usize,    // 爆炸等级
    pub show_prop: bool,      // 是否出现道具
}

/// 标记需要销毁的实体（由 despawn_marked_entities 系统统一清理）
#[derive(Component)]
pub struct ShouldDespawn;

/// 碰撞层枚举
#[derive(PhysicsLayer, Default, Clone, Copy, Debug)]
pub enum GameLayer {
    #[default]
    Default,
    Wall,     // 墙体
    River,    // 河流
    Player,   // 玩家
    Enemy,    // 敌人
    Bullet,   // 炮弹
    Boundary, // 边界
    Prop,     // 道具
}

/// 定义碰撞规则组
pub mod collision_groups {
    use super::*;

    /// 玩家：与玩家、敌人、子弹、墙体、道具碰撞
    pub fn player() -> CollisionLayers {
        CollisionLayers::new(
            GameLayer::Player,
            [
                GameLayer::Player,
                GameLayer::Enemy,
                GameLayer::Bullet,
                GameLayer::Wall,
                GameLayer::Prop,
            ],
        )
    }

    /// 敌人：与敌人、玩家、子弹、墙体、道具碰撞
    pub fn enemy() -> CollisionLayers {
        CollisionLayers::new(
            GameLayer::Enemy,
            [
                GameLayer::Enemy,
                GameLayer::Player,
                GameLayer::Bullet,
                GameLayer::Wall,
                GameLayer::Prop,
            ],
        )
    }

    /// 边界/石头/墙体/堡垒：与玩家、敌人、子弹碰撞
    pub fn wall() -> CollisionLayers {
        CollisionLayers::new(
            GameLayer::Wall,
            [GameLayer::Player, GameLayer::Enemy, GameLayer::Bullet],
        )
    }

    /// 河流：与玩家、敌人碰撞
    pub fn river() -> CollisionLayers {
        CollisionLayers::new(GameLayer::River, [GameLayer::Player, GameLayer::Enemy])
    }

    /// 子弹：与玩家、敌人、墙体、子弹、边界碰撞
    pub fn bullet() -> CollisionLayers {
        CollisionLayers::new(
            GameLayer::Bullet,
            [
                GameLayer::Player,
                GameLayer::Enemy,
                GameLayer::Wall,
                GameLayer::Bullet,
                GameLayer::Boundary,
            ],
        )
    }

    /// 道具：道具与玩家（可以配置成敌人也能拾取道具使用道具效果）
    pub fn prop() -> CollisionLayers {
        CollisionLayers::new(GameLayer::Prop, [GameLayer::Player, GameLayer::Enemy])
    }
}

/// 添加道具碰撞相关组件
pub fn add_prop_collision() -> impl Bundle {
    (
        RigidBody::Static,
        Collider::rectangle(16., 16.),
        Sensor,
        CollisionEventsEnabled,
        Friction::new(0.),
        Restitution::ZERO,
        collision_groups::prop(),
    )
}

/// 添加墙体碰撞相关组件
pub fn add_wall_collision(width: f32, height: f32, collision_type: &str) -> impl Bundle {
    let collision_groups = match collision_type {
        "river" => collision_groups::river(),
        _ => collision_groups::wall(),
    };
    (
        RigidBody::Static,
        Collider::rectangle(width, height),
        Friction::new(0.),
        Restitution::ZERO,
        collision_groups,
    )
}

/// 坦克物理标记（使用 move-and-slide 移动，避免相互推挤）
#[derive(Component)]
struct TankPhysics;

/// 添加坦克碰撞相关组件
pub fn add_tank_collision(collision_type: &str) -> impl Bundle {
    let collision_groups = match collision_type {
        "player" => collision_groups::player(),
        _ => collision_groups::enemy(),
    };
    (
        TankPhysics,
        RigidBody::Kinematic,
        Collider::rectangle(14., 15.),
        CustomPositionIntegration,
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity(Vec2::new(0., 0.)),
        collision_groups,
    )
}

/// 移除坦克物理碰撞（玩家被摧毁后调用，避免残留碰撞体）
pub fn strip_tank_physics(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<(
        TankPhysics,
        RigidBody,
        Collider,
        CustomPositionIntegration,
        LockedAxes,
        LinearVelocity,
        CollisionLayers,
    )>();
}

/// 坦克仅沿四向轴移动；若 move-and-slide 产生垂直于意图方向的位移（侧滑），则视为被挡并停车。
const TANK_AXIS_SLIDE_THRESHOLD: f32 = 0.01;

fn tank_axis_slide(intended: Vec2, displacement: Vec2) -> bool {
    if intended.length_squared() < f32::EPSILON {
        return false;
    }
    if intended.x.abs() > intended.y.abs() {
        displacement.y.abs() > TANK_AXIS_SLIDE_THRESHOLD
    } else {
        displacement.x.abs() > TANK_AXIS_SLIDE_THRESHOLD
    }
}

/// 坦克 move-and-slide 移动：阻挡墙体与其他坦克，但不产生物理推挤
fn tank_move_and_slide(
    mut query: Query<(Entity, &mut Transform, &mut LinearVelocity, &Collider), With<TankPhysics>>,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
) {
    for (entity, mut transform, mut velocity, collider) in &mut query {
        let intended_velocity = velocity.0;
        let previous_position = transform.translation.xy();

        let MoveAndSlideOutput {
            position,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            previous_position.adjust_precision(),
            Rotation::from(transform.rotation).as_radians(),
            intended_velocity.adjust_precision(),
            time.delta(),
            &MoveAndSlideConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |_| MoveAndSlideHitResponse::Accept,
        );

        let new_position = Vec2::new(position.x, position.y);
        let displacement = new_position - previous_position;

        if tank_axis_slide(intended_velocity, displacement) {
            velocity.0 = Vec2::ZERO;
            continue;
        }

        transform.translation.x = new_position.x;
        transform.translation.y = new_position.y;
        velocity.0 = projected_velocity;
    }
}

/// 添加炮弹碰撞相关组件
pub fn add_bullet_collision(vel: Vec2) -> impl Bundle {
    (
        RigidBody::Dynamic,
        Collider::rectangle(10., 4.),
        Friction::new(0.),
        Restitution::ZERO,
        LinearVelocity(vel),
        LockedAxes::ROTATION_LOCKED,
        CollisionEventsEnabled, // 启用碰撞事件
        collision_groups::bullet(),
    )
}

/// 定义炮弹碰撞钩子参数
#[derive(SystemParam)]
pub struct BulletCollisionHooks<'w, 's> {
    bullet_query: Query<'w, 's, &'static BulletInfo>,
}

/// 检测是否自己炮弹
impl CollisionHooks for BulletCollisionHooks<'_, '_> {
    fn filter_pairs(&self, collider1: Entity, collider2: Entity, _commands: &mut Commands) -> bool {
        match (
            self.bullet_query.get(collider1),
            self.bullet_query.get(collider2),
        ) {
            (Ok(b1), Ok(b2)) => {
                // 两发炮弹：只允许不同阵营碰撞（玩家炮弹 VS 敌人炮弹）
                b1.horde != b2.horde
            }
            (Ok(bullet), Err(_)) => {
                // collider1 是炮弹：过滤掉与发射者的碰撞
                collider2 != bullet.entity
            }
            (Err(_), Ok(bullet)) => {
                // collider2 是炮弹：过滤掉与发射者的碰撞
                collider1 != bullet.entity
            }
            (Err(_), Err(_)) => true,
        }
    }
}
