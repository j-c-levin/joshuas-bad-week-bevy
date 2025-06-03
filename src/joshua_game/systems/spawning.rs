//! Enemy spawning systems

use bevy::prelude::*;
use rand::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{
            Card, CollisionBox, Damage, GameEntity, Health, Joel, Kezia, MaxSpeed, MoveInDirection,
            MoveTowardsPoint, OffScreenCleanup, Player, PlayerTarget, RotateTowardsTarget,
            SpawnSide, Timer, TurnRate, Velocity,
        },
        config::GameConfig,
        events::{CardSpawnEvent, EnemySpawnEvent, EnemyType},
        resources::{DifficultyState, GameState},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_enemies_system,
            handle_enemy_spawn_events,
            handle_card_spawn_events,
            spawn_initial_player_ecs,
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

    // Update single spawn timer
    difficulty.spawn_timer += delta;

    // When timer triggers, randomly choose enemy type
    if difficulty.spawn_timer >= difficulty.current_spawn_interval {
        difficulty.spawn_timer = 0.0;

        let mut rng = thread_rng();
        let (spawn_pos, spawn_side) = generate_spawn_position(&config);

        // Randomly choose between Kezia and Joel with equal probability
        let enemy_type = if rng.gen_bool(0.5) {
            EnemyType::Kezia
        } else {
            EnemyType::Joel
        };

        spawn_events.write(EnemySpawnEvent {
            enemy_type,
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
                spawn_kezia_ecs(&mut commands, event.position, event.spawn_side, &config);
            }
            EnemyType::Joel => {
                spawn_joel_ecs(&mut commands, event.position, event.spawn_side, &config);
            }
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

/// Generate a random spawn position just off-screen
fn generate_spawn_position(config: &GameConfig) -> (Vec2, SpawnSide) {
    let mut rng = thread_rng();
    let side = rng.gen_range(0..4);
    let spawn_distance = config.spawn_distance;

    let screen_half_width = config.screen_width / 2.0;
    let screen_half_height = config.screen_height / 2.0;

    match side {
        0 => {
            // Top
            let x = rng.gen_range(-screen_half_width..screen_half_width);
            let y = screen_half_height + spawn_distance;
            (Vec2::new(x, y), SpawnSide::Top)
        }
        1 => {
            // Right
            let x = screen_half_width + spawn_distance;
            let y = rng.gen_range(-screen_half_height..screen_half_height);
            (Vec2::new(x, y), SpawnSide::Right)
        }
        2 => {
            // Bottom
            let x = rng.gen_range(-screen_half_width..screen_half_width);
            let y = -screen_half_height - spawn_distance;
            (Vec2::new(x, y), SpawnSide::Bottom)
        }
        _ => {
            // Left
            let x = -screen_half_width - spawn_distance;
            let y = rng.gen_range(-screen_half_height..screen_half_height);
            (Vec2::new(x, y), SpawnSide::Left)
        }
    }
}

/// Calculate Joel's target position (approach distance from spawn position towards screen)
fn calculate_joel_target_position(
    spawn_position: Vec2,
    spawn_side: SpawnSide,
    config: &GameConfig,
) -> Vec2 {
    let approach_distance = config.spawn_distance + config.joel_approach_distance;

    match spawn_side {
        SpawnSide::Top => spawn_position + Vec2::new(0.0, -approach_distance), // Move down from spawn
        SpawnSide::Right => spawn_position + Vec2::new(-approach_distance, 0.0), // Move left from spawn
        SpawnSide::Bottom => spawn_position + Vec2::new(0.0, approach_distance), // Move up from spawn
        SpawnSide::Left => spawn_position + Vec2::new(approach_distance, 0.0), // Move right from spawn
    }
}

/// Spawn a Kezia enemy using new ECS components
fn spawn_kezia_ecs(
    commands: &mut Commands,
    position: Vec2,
    spawn_side: SpawnSide,
    config: &GameConfig,
) {
    // Calculate initial rotation and velocity based on spawn side to enter screen properly
    let (initial_rotation, initial_velocity) = match spawn_side {
        SpawnSide::Top => {
            // Coming from top, face downward
            let rotation = -std::f32::consts::PI / 2.0; // -90 degrees (pointing down)
            let velocity = Vec2::new(0.0, -config.kezia_speed);
            (rotation, velocity)
        }
        SpawnSide::Right => {
            // Coming from right, face leftward
            let rotation = std::f32::consts::PI; // 180 degrees (pointing left)
            let velocity = Vec2::new(-config.kezia_speed, 0.0);
            (rotation, velocity)
        }
        SpawnSide::Bottom => {
            // Coming from bottom, face upward
            let rotation = std::f32::consts::PI / 2.0; // 90 degrees (pointing up)
            let velocity = Vec2::new(0.0, config.kezia_speed);
            (rotation, velocity)
        }
        SpawnSide::Left => {
            // Coming from left, face rightward
            let rotation = 0.0; // 0 degrees (pointing right)
            let velocity = Vec2::new(config.kezia_speed, 0.0);
            (rotation, velocity)
        }
    };

    let entity = commands
        .spawn((
            Name::new("Kezia"),
            Transform {
                translation: position.extend(0.0),
                rotation: Quat::from_rotation_z(initial_rotation),
                ..default()
            },
            Kezia::default(), // Keep marker component for rendering system
            Velocity(initial_velocity),
            MaxSpeed(config.kezia_speed),
            TurnRate(config.kezia_turn_rate),
            RotateTowardsTarget::new(0.0),
            Timer::new(config.kezia_tracking_duration, false),
        ))
        .id();

    // Add remaining components
    commands.entity(entity).insert((
        MoveInDirection, // Continue moving straight after tracking
        CollisionBox::new(Vec2::new(config.kezia_width, config.kezia_height)),
        Damage,
        OffScreenCleanup,
        GameEntity,
    ));

    info!(
        "Kezia ECS spawned at {:?} from side {:?}",
        position, spawn_side
    );
}

/// Spawn a Joel enemy using new ECS components
fn spawn_joel_ecs(
    commands: &mut Commands,
    position: Vec2,
    spawn_side: SpawnSide,
    config: &GameConfig,
) {
    let target_position = calculate_joel_target_position(position, spawn_side, config);

    // Calculate initial rotation and velocity to move perpendicular to spawn side
    let (initial_rotation, initial_velocity) = match spawn_side {
        SpawnSide::Top => {
            // Coming from top, face downward initially
            let rotation = -std::f32::consts::PI / 2.0; // -90 degrees (pointing down)
            let velocity = Vec2::new(0.0, -config.joel_speed);
            (rotation, velocity)
        }
        SpawnSide::Right => {
            // Coming from right, face leftward initially
            let rotation = std::f32::consts::PI; // 180 degrees (pointing left)
            let velocity = Vec2::new(-config.joel_speed, 0.0);
            (rotation, velocity)
        }
        SpawnSide::Bottom => {
            // Coming from bottom, face upward initially
            let rotation = std::f32::consts::PI / 2.0; // 90 degrees (pointing up)
            let velocity = Vec2::new(0.0, config.joel_speed);
            (rotation, velocity)
        }
        SpawnSide::Left => {
            // Coming from left, face rightward initially
            let rotation = 0.0; // 0 degrees (pointing right)
            let velocity = Vec2::new(config.joel_speed, 0.0);
            (rotation, velocity)
        }
    };

    let entity = commands
        .spawn((
            Name::new("Joel"),
            Transform {
                translation: position.extend(0.0),
                rotation: Quat::from_rotation_z(initial_rotation),
                ..default()
            },
            Joel::new(spawn_side, position, target_position), // Keep marker component for rendering system
            Velocity(initial_velocity),
            MaxSpeed(config.joel_speed),
            TurnRate(config.joel_turn_rate),
            MoveTowardsPoint::new(target_position, 5.0),
        ))
        .id();

    // Add remaining components
    commands.entity(entity).insert((
        Timer::new(config.joel_tracking_duration, false),
        CollisionBox::new(Vec2::new(config.joel_width, config.joel_height)),
        Damage,
        OffScreenCleanup,
        GameEntity,
    ));

    info!(
        "Joel ECS spawned at {:?} from side {:?}",
        position, spawn_side
    );
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
        Card,
        Velocity(direction * config.card_speed),
        MoveInDirection,
        CollisionBox::new(Vec2::new(config.card_width, config.card_height)),
        Damage,
        OffScreenCleanup,
        GameEntity,
    ));

    info!(
        "Card ECS spawned at {:?} in direction {:?}",
        position, direction
    );
}
