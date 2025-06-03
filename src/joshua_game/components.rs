//! ECS Components for Joshua's Bad Week entities

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>()
        .register_type::<Health>()
        .register_type::<Movement>()
        .register_type::<CollisionBox>()
        .register_type::<Kezia>()
        .register_type::<Joel>()
        .register_type::<Card>()
        .register_type::<JoelState>()
        .register_type::<SpawnSide>()
        // New ECS-friendly components
        .register_type::<Velocity>()
        .register_type::<MaxSpeed>()
        .register_type::<TurnRate>()
        .register_type::<TrackTarget>()
        .register_type::<RotateTowardsTarget>()
        .register_type::<MoveTowardsPoint>()
        .register_type::<ProjectileLauncher>()
        .register_type::<Timer>()
        .register_type::<LifetimeTimer>()
        .register_type::<KeziaState>()
        .register_type::<NewJoelState>();
}

// ==================== Player Components ====================

#[derive(Component, Reflect)]
pub struct Player {
    pub trail_timer: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self { trail_timer: 0.0 }
    }
}

// ==================== Marker Components ====================

#[derive(Component, Reflect)]
pub struct PlayerTarget;

#[derive(Component, Reflect)]
pub struct MoveInDirection;

// ==================== Core Components ====================

#[derive(Component, Reflect)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }
    
    pub fn take_damage(&mut self, damage: i32) {
        self.current = (self.current - damage).max(0);
    }
    
    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
    
    pub fn health_ratio(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}

#[derive(Component, Reflect)]
pub struct Movement {
    pub velocity: Vec2,
    pub speed: f32,
    pub rotation: f32,
    pub turn_rate: f32,
}

impl Movement {
    pub fn new(speed: f32, turn_rate: f32) -> Self {
        Self {
            velocity: Vec2::ZERO,
            speed,
            rotation: 0.0,
            turn_rate,
        }
    }
}

#[derive(Component, Reflect)]
pub struct CollisionBox {
    pub size: Vec2,
}

impl CollisionBox {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
    
    pub fn get_rect(&self, position: Vec2) -> (Vec2, Vec2) {
        let half_size = self.size / 2.0;
        (position - half_size, position + half_size)
    }
}

// ==================== New ECS Movement Components ====================

#[derive(Component, Reflect)]
pub struct Velocity(pub Vec2);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

#[derive(Component, Reflect)]
pub struct MaxSpeed(pub f32);

#[derive(Component, Reflect)]
pub struct TurnRate(pub f32);

// ==================== New ECS Behavior Components ====================

#[derive(Component, Reflect)]
pub struct TrackTarget {
    pub target_entity: Option<Entity>,
}

impl Default for TrackTarget {
    fn default() -> Self {
        Self {
            target_entity: None,
        }
    }
}

#[derive(Component, Reflect)]
pub struct RotateTowardsTarget {
    pub target_entity: Option<Entity>,
    pub offset_angle: f32,
}

impl RotateTowardsTarget {
    pub fn new(offset_angle: f32) -> Self {
        Self {
            target_entity: None,
            offset_angle,
        }
    }
}

impl Default for RotateTowardsTarget {
    fn default() -> Self {
        Self::new(0.0)
    }
}

#[derive(Component, Reflect)]
pub struct MoveTowardsPoint {
    pub target_position: Vec2,
    pub stop_distance: f32,
}

impl MoveTowardsPoint {
    pub fn new(target_position: Vec2, stop_distance: f32) -> Self {
        Self {
            target_position,
            stop_distance,
        }
    }
}

#[derive(Component, Reflect)]
pub struct ProjectileLauncher {
    pub fire_rate: f32,
    pub timer: f32,
    pub projectile_speed: f32,
    pub damage: i32,
    pub target_entity: Option<Entity>,
}

impl ProjectileLauncher {
    pub fn new(fire_rate: f32, projectile_speed: f32, damage: i32) -> Self {
        Self {
            fire_rate,
            timer: 0.0,
            projectile_speed,
            damage,
            target_entity: None,
        }
    }
}

#[derive(Component, Reflect)]
pub struct Timer {
    pub duration: f32,
    pub elapsed: f32,
    pub repeating: bool,
}

impl Timer {
    pub fn new(duration: f32, repeating: bool) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating,
        }
    }
    
    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed = 0.0;
            }
            true
        } else {
            false
        }
    }
    
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
    
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}

#[derive(Component, Reflect)]
pub struct LifetimeTimer {
    pub remaining: f32,
}

impl LifetimeTimer {
    pub fn new(lifetime: f32) -> Self {
        Self {
            remaining: lifetime,
        }
    }
}

// ==================== New ECS State Components ====================

#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum KeziaState {
    Tracking,
    MovingStraight,
}

impl Default for KeziaState {
    fn default() -> Self {
        KeziaState::Tracking
    }
}

#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum NewJoelState {
    Approaching,
    Tracking,
    Retreating,
}

impl Default for NewJoelState {
    fn default() -> Self {
        NewJoelState::Approaching
    }
}

// ==================== Legacy Enemy Components ====================

#[derive(Component, Reflect)]
pub struct Kezia {
    pub tracking_timer: f32,
    pub is_tracking: bool,
}

impl Default for Kezia {
    fn default() -> Self {
        Self {
            tracking_timer: 0.0,
            is_tracking: true,
        }
    }
}

#[derive(Component, Reflect)]
pub struct Joel {
    pub state: JoelState,
    pub state_timer: f32,
    pub card_fire_timer: f32,
    pub spawn_side: SpawnSide,
    pub target_position: Vec2,
    pub spawn_position: Vec2,
    pub is_charging: bool,
    pub charge_timer: f32,
}

impl Joel {
    pub fn new(spawn_side: SpawnSide, spawn_position: Vec2, target_position: Vec2) -> Self {
        Self {
            state: JoelState::Approaching,
            state_timer: 0.0,
            card_fire_timer: 0.0,
            spawn_side,
            target_position,
            spawn_position,
            is_charging: false,
            charge_timer: 0.0,
        }
    }
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum JoelState {
    Approaching,
    Tracking,
    Retreating,
}

impl Default for JoelState {
    fn default() -> Self {
        JoelState::Approaching
    }
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum SpawnSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl Default for SpawnSide {
    fn default() -> Self {
        SpawnSide::Top
    }
}

// ==================== Projectile Components ====================

#[derive(Component, Reflect)]
pub struct Card {
    pub damage: i32,
    pub lifetime: f32,
}

impl Card {
    pub fn new(damage: i32, lifetime: f32) -> Self {
        Self { damage, lifetime }
    }
}

// ==================== Utility Components ====================

#[derive(Component)]
pub struct OffScreenCleanup;

#[derive(Component)]
pub struct GameEntity; // Marker for entities that should be cleaned up when the game ends 