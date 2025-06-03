//! Enemy spawning systems

use bevy::prelude::*;
use rand::prelude::*;

use crate::{
    joshua_game::{
        components::{
            Player, Kezia, Joel, Card, Movement, CollisionBox, Health, OffScreenCleanup, 
            GameEntity, SpawnSide,
            PlayerTarget, Velocity, MaxSpeed, TurnRate, TrackTarget, RotateTowardsTarget,
            MoveTowardsPoint, ProjectileLauncher, Timer, LifetimeTimer, KeziaState, NewJoelState,
            MoveInDirection,
        },
        config::GameConfig,
        resources::{GameState, DifficultyState},
        events::{EnemySpawnEvent, EnemyType, CardSpawnEvent},
    },
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_enemies_system,
            handle_enemy_spawn_events,
            handle_card_spawn_events,
            spawn_initial_player_ecs,
            update_card_lifetime,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

/// Spawns the player using new ECS components
fn spawn_initial_player_ecs(
    mut commands: Commands,
    config: Res<GameConfig>,
    player_query: Query<&PlayerTarget>,
    game_state: Res<GameState>,
) {
    // Only spawn player once and when game is active
    if !player_query.is_empty() || !game_state.is_game_active() {
        return;
    }

    commands.spawn((
        Name::new("Player"),
        Transform::from_translation(Vec3::ZERO), // Spawn at center like the original
        Player::default(),
        PlayerTarget, // Mark as target for enemies
        Health::new(config.initial_health),
        Velocity::default(),
        MaxSpeed(config.player_speed),
        CollisionBox::new(Vec2::splat(config.player_size)),
        GameEntity,
    ));

    info!("Player spawned at center");
}

/// Main enemy spawning logic with difficulty scaling
fn spawn_enemies_system(
    time: Res<Time>,
    mut difficulty: ResMut<DifficultyState>,
    mut spawn_events: EventWriter<EnemySpawnEvent>,
    game_state: Res<GameState>,
    config: Res<GameConfig>,
) {
    if !game_state.is_game_active() {
        return;
    }

    let delta = time.delta_secs();

    // Update spawn timers
    difficulty.kezia_spawn_timer += delta;
    difficulty.joel_spawn_timer += delta;

    // Spawn Kezia enemies
    if difficulty.kezia_spawn_timer >= difficulty.current_kezia_interval {
        difficulty.kezia_spawn_timer = 0.0;
        
        let (spawn_pos, spawn_side) = generate_spawn_position(&config);
        spawn_events.write(EnemySpawnEvent {
            enemy_type: EnemyType::Kezia,
            position: spawn_pos,
            spawn_side,
        });
    }

    // Spawn Joel enemies (less frequently)
    if difficulty.joel_spawn_timer >= difficulty.current_joel_interval {
        difficulty.joel_spawn_timer = 0.0;
        
        let (spawn_pos, spawn_side) = generate_spawn_position(&config);
        spawn_events.write(EnemySpawnEvent {
            enemy_type: EnemyType::Joel,
            position: spawn_pos,
            spawn_side,
        });
    }
}

/// Handle enemy spawn events
fn handle_enemy_spawn_events(
    mut commands: Commands,
    mut events: EventReader<EnemySpawnEvent>,
    config: Res<GameConfig>,
) {
    for event in events.read() {
        match event.enemy_type {
            EnemyType::Kezia => {
                spawn_kezia_ecs(&mut commands, event.position, &config);
            },
            EnemyType::Joel => {
                spawn_joel_ecs(&mut commands, event.position, event.spawn_side, &config);
            },
        }
    }
}

/// Handle card spawn events
fn handle_card_spawn_events(
    mut commands: Commands,
    mut events: EventReader<CardSpawnEvent>,
    config: Res<GameConfig>,
) {
    for event in events.read() {
        spawn_card_ecs(&mut commands, event.position, event.direction, &config);
    }
}

/// Update card lifetime and despawn expired cards
fn update_card_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut card_query: Query<(Entity, &mut Card)>,
) {
    let delta = time.delta_secs();
    
    for (entity, mut card) in &mut card_query {
        card.lifetime -= delta;
        
        if card.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn a Kezia enemy
fn spawn_kezia(commands: &mut Commands, position: Vec2, config: &GameConfig) {
    commands.spawn((
        Name::new("Kezia"),
        Transform::from_translation(position.extend(0.0)),
        Kezia::default(),
        Movement::new(config.kezia_speed, config.kezia_turn_rate),
        CollisionBox::new(Vec2::new(config.kezia_width, config.kezia_height)),
        OffScreenCleanup,
        GameEntity,
    ));

    info!("Kezia spawned at {:?}", position);
}

/// Spawn a Joel enemy
fn spawn_joel(commands: &mut Commands, position: Vec2, spawn_side: SpawnSide, config: &GameConfig) {
    let target_position = calculate_joel_target_position(spawn_side, config);
    
    commands.spawn((
        Name::new("Joel"),
        Transform::from_translation(position.extend(0.0)),
        Joel::new(spawn_side, position, target_position),
        Movement::new(config.joel_speed, config.joel_turn_rate),
        CollisionBox::new(Vec2::new(config.joel_width, config.joel_height)),
        OffScreenCleanup,
        GameEntity,
    ));

    info!("Joel spawned at {:?}", position);
}

/// Spawn a card projectile
fn spawn_card(commands: &mut Commands, position: Vec2, direction: Vec2, config: &GameConfig) {
    let mut movement = Movement::new(config.card_speed, 0.0);
    movement.velocity = direction * config.card_speed;
    movement.rotation = direction.y.atan2(direction.x);
    
    commands.spawn((
        Name::new("Card"),
        Transform::from_translation(position.extend(0.0)),
        Card::new(config.card_damage, 5.0), // 5 second lifetime
        movement,
        CollisionBox::new(Vec2::new(config.card_width, config.card_height)),
        OffScreenCleanup,
        GameEntity,
    ));

    info!("Card spawned at {:?} in direction {:?}", position, direction);
}

/// Generate a random spawn position just off-screen
fn generate_spawn_position(config: &GameConfig) -> (Vec2, SpawnSide) {
    let mut rng = thread_rng();
    let side = rng.gen_range(0..4);
    let spawn_distance = config.kezia_spawn_distance;
    
    let screen_half_width = config.screen_width / 2.0;
    let screen_half_height = config.screen_height / 2.0;

    match side {
        0 => { // Top
            let x = rng.gen_range(-screen_half_width..screen_half_width);
            let y = screen_half_height + spawn_distance;
            (Vec2::new(x, y), SpawnSide::Top)
        },
        1 => { // Right
            let x = screen_half_width + spawn_distance;
            let y = rng.gen_range(-screen_half_height..screen_half_height);
            (Vec2::new(x, y), SpawnSide::Right)
        },
        2 => { // Bottom
            let x = rng.gen_range(-screen_half_width..screen_half_width);
            let y = -screen_half_height - spawn_distance;
            (Vec2::new(x, y), SpawnSide::Bottom)
        },
        _ => { // Left
            let x = -screen_half_width - spawn_distance;
            let y = rng.gen_range(-screen_half_height..screen_half_height);
            (Vec2::new(x, y), SpawnSide::Left)
        },
    }
}

/// Calculate Joel's target position (approach distance from screen edge)
fn calculate_joel_target_position(spawn_side: SpawnSide, config: &GameConfig) -> Vec2 {
    let approach_distance = config.joel_approach_distance;
    let screen_half_width = config.screen_width / 2.0;
    let screen_half_height = config.screen_height / 2.0;

    match spawn_side {
        SpawnSide::Top => Vec2::new(0.0, screen_half_height - approach_distance),
        SpawnSide::Right => Vec2::new(screen_half_width - approach_distance, 0.0),
        SpawnSide::Bottom => Vec2::new(0.0, -screen_half_height + approach_distance),
        SpawnSide::Left => Vec2::new(-screen_half_width + approach_distance, 0.0),
    }
}

/// Spawn a Kezia enemy using new ECS components
fn spawn_kezia_ecs(commands: &mut Commands, position: Vec2, config: &GameConfig) {
    commands.spawn((
        Name::new("Kezia"),
        Transform::from_translation(position.extend(0.0)),
        Kezia::default(), // Keep marker component for rendering system
        Velocity::default(),
        MaxSpeed(config.kezia_speed),
        TurnRate(config.kezia_turn_rate),
        TrackTarget::default(),
        RotateTowardsTarget::default(),
        KeziaState::default(),
        Timer::new(config.kezia_tracking_duration, false),
        MoveInDirection, // Continue moving straight after tracking
        CollisionBox::new(Vec2::new(config.kezia_width, config.kezia_height)),
        OffScreenCleanup,
        GameEntity,
    ));

    info!("Kezia ECS spawned at {:?}", position);
}

/// Spawn a Joel enemy using new ECS components
fn spawn_joel_ecs(commands: &mut Commands, position: Vec2, spawn_side: SpawnSide, config: &GameConfig) {
    let target_position = calculate_joel_target_position(spawn_side, config);
    
    commands.spawn((
        Name::new("Joel"),
        Transform::from_translation(position.extend(0.0)),
        Joel::new(spawn_side, position, target_position), // Keep marker component for rendering system
        Velocity::default(),
        MaxSpeed(config.joel_speed),
        TurnRate(config.joel_turn_rate),
        NewJoelState::default(),
        MoveTowardsPoint::new(target_position, 5.0),
        RotateTowardsTarget::new(std::f32::consts::PI / 2.0), // Joel faces perpendicular to player
        ProjectileLauncher::new(config.joel_card_fire_rate, config.card_speed, config.card_damage),
        Timer::new(config.joel_tracking_duration, false),
        CollisionBox::new(Vec2::new(config.joel_width, config.joel_height)),
        OffScreenCleanup,
        GameEntity,
    ));

    info!("Joel ECS spawned at {:?}", position);
}

/// Spawn a card projectile using new ECS components
fn spawn_card_ecs(commands: &mut Commands, position: Vec2, direction: Vec2, config: &GameConfig) {
    let rotation = direction.y.atan2(direction.x);
    
    commands.spawn((
        Name::new("Card"),
        Transform {
            translation: position.extend(0.0),
            rotation: Quat::from_rotation_z(rotation),
            ..default()
        },
        Card::new(config.card_damage, 5.0), // Keep component for rendering system
        Velocity(direction * config.card_speed),
        MoveInDirection,
        LifetimeTimer::new(5.0),
        CollisionBox::new(Vec2::new(config.card_width, config.card_height)),
        OffScreenCleanup,
        GameEntity,
    ));

    info!("Card ECS spawned at {:?} in direction {:?}", position, direction);
} 