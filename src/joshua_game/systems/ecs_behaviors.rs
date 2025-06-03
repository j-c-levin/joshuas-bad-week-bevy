//! Modular ECS behavior systems for entity behaviors

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{
            Joel, Kezia, MaxSpeed, MoveInDirection, MoveTowardsPoint,
            PlayerTarget, ProjectileLauncher, RotateTowardsTarget, Timer, TurnRate,
            Velocity,
        },
        config::GameConfig,
        events::CardSpawnEvent,
    },
};

/// System sets for organizing ECS behavior execution phases
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum BehaviorPhase {
    /// Calculate and set velocities
    UpdateVelocity,
    /// Apply physics (velocity to position)
    ApplyPhysics,
    /// Other effects that don't affect movement
    Effects,
}

pub(super) fn plugin(app: &mut App) {
    // Configure the execution order of our behavior phases
    app.configure_sets(
        Update,
        (
            BehaviorPhase::UpdateVelocity,
            BehaviorPhase::ApplyPhysics,
            BehaviorPhase::Effects,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );

    app.add_systems(
        Update,
        (
            // Phase 1: Systems that calculate and set velocity (order matters within this phase)
            rotate_towards_target_system,
            move_in_direction_system,
            move_towards_point_system,
            kezia_system,
            joel_start_tracking_system,
            joel_finish_tracking_system,
            timer_tick_system,
        )
            .chain()
            .in_set(BehaviorPhase::UpdateVelocity),
    );

    app.add_systems(
        Update,
        // Phase 2: Apply physics
        velocity_system.in_set(BehaviorPhase::ApplyPhysics),
    );

    app.add_systems(
        Update,
        (
            // Phase 3: Other effects that don't affect movement
            projectile_launcher_system,
        )
            .in_set(BehaviorPhase::Effects),
    );
}

// ==================== Core Movement Systems ====================

/// Apply velocity to entity positions
fn velocity_system(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();
    }
}

/// For entities that should continue moving in their current direction (like projectiles)
fn move_in_direction_system(
    mut query: Query<(&mut Velocity, &Transform, &MaxSpeed), With<MoveInDirection>>,
) {
    for (mut velocity, transform, max_speed) in &mut query {
        let rotation = transform.rotation.to_euler(EulerRot::ZYX).0;
        velocity.0 = Vec2::new(rotation.cos(), rotation.sin()) * max_speed.0;
    }
}

// ==================== Behavior Systems ====================

/// Rotate entities towards their target (only during tracking state for Kezia)
fn rotate_towards_target_system(
    time: Res<Time>,
    player_query: Query<&Transform, With<PlayerTarget>>,
    mut rotate_query: Query<
        (&RotateTowardsTarget, &mut Transform, &TurnRate),
        Without<PlayerTarget>,
    >,
) {
    let delta = time.delta_secs();

    // Get the player position
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.xy();

    // Rotate all entities with RotateTowardsTarget component towards the player
    for (rotate_target, mut transform, turn_rate) in &mut rotate_query {
        let current_pos = transform.translation.xy();
        let direction = player_pos - current_pos;

        if direction.length() > 0.1 {
            let target_rotation = direction.y.atan2(direction.x) + rotate_target.offset_angle;
            let current_rotation = transform.rotation.to_euler(EulerRot::ZYX).0;

            // Calculate shortest rotation path (prevents 180° flipping)
            let mut rotation_diff = target_rotation - current_rotation;

            // Normalize to [-π, π] range
            while rotation_diff > std::f32::consts::PI {
                rotation_diff -= 2.0 * std::f32::consts::PI;
            }
            while rotation_diff < -std::f32::consts::PI {
                rotation_diff += 2.0 * std::f32::consts::PI;
            }

            // Apply rotation with speed limit
            let max_rotation = turn_rate.0 * delta;
            let actual_rotation = rotation_diff.clamp(-max_rotation, max_rotation);

            let new_rotation = current_rotation + actual_rotation;
            transform.rotation = Quat::from_rotation_z(new_rotation);
        }
    }
}

/// Move entities towards a specific point and remove component when reached
fn move_towards_point_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MoveTowardsPoint, &Transform, &mut Velocity, &MaxSpeed)>,
) {
    for (entity, move_towards, transform, mut velocity, max_speed) in &mut query {
        let current_pos = transform.translation.xy();
        let distance = current_pos.distance(move_towards.target_position);

        if distance <= move_towards.stop_distance {
            velocity.0 = Vec2::ZERO;
            // Remove the component when destination is reached
            commands.entity(entity).remove::<MoveTowardsPoint>();
        } else {
            let direction = (move_towards.target_position - current_pos).normalize_or_zero();
            velocity.0 = direction * max_speed.0;
        }
    }
}

/// Handle Joel entities that just finished moving to their target position and start tracking
fn joel_start_tracking_system(
    mut commands: Commands,
    mut removed_move_towards: RemovedComponents<MoveTowardsPoint>,
    mut joel_query: Query<&mut Timer, With<Joel>>,
    config: Res<GameConfig>,
) {
    for entity in removed_move_towards.read() {
        // Check if this entity is a Joel and in the appropriate state
        if let Ok(mut timer) = joel_query.get_mut(entity) {
            info!("Joel finished moving to target, transitioning to Tracking");
            
            // Reset timer for tracking duration
            timer.reset();
            
            // Add RotateTowardsTarget component when Joel finishes approaching
            commands
                .entity(entity)
                .insert(RotateTowardsTarget::new(std::f32::consts::PI / 2.0));
            
            // Add ProjectileLauncher component to start firing cards
            commands
                .entity(entity)
                .insert(ProjectileLauncher::new(config.joel_card_fire_rate, config.card_speed));
        }
    }
}

/// Handle projectile launching
fn projectile_launcher_system(
    time: Res<Time>,
    player_query: Query<&Transform, With<PlayerTarget>>,
    mut launcher_query: Query<(&mut ProjectileLauncher, &Transform), Without<PlayerTarget>>,
    mut spawn_events: EventWriter<CardSpawnEvent>,
) {
    let delta = time.delta_secs();

    // Get the player position
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.xy();

    for (mut launcher, transform) in &mut launcher_query {
        launcher.timer += delta;

        if launcher.timer >= launcher.fire_rate {
            let direction = (player_pos - transform.translation.xy()).normalize_or_zero();

            spawn_events.write(CardSpawnEvent {
                position: transform.translation.xy(),
                direction,
            });

            launcher.timer = 0.0;
        }
    }
}

// ==================== Utility Systems ====================

/// Tick timers for all entities
fn timer_tick_system(time: Res<Time>, mut timer_query: Query<&mut Timer>) {
    let delta = time.delta_secs();

    for mut timer in &mut timer_query {
        timer.tick(delta);
    }
}

// ==================== State-Specific Systems ====================

/// Handle Kezia state transitions
fn kezia_system(mut commands: Commands, mut query: Query<(Entity, &mut Timer), With<Kezia>>) {
    for (entity, timer) in &mut query {
        if timer.is_finished() {
            commands.entity(entity).remove::<RotateTowardsTarget>();
        }
    }
}

/// Handle Joel entities that just finished tracking and need to retreat
fn joel_finish_tracking_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Timer,
        &Transform,
        &Joel,
        &mut Velocity,
        &MaxSpeed,
    ), (With<RotateTowardsTarget>, Without<MoveTowardsPoint>)>,
    _config: Res<GameConfig>,
) {
    for (
        entity,
        timer,
        transform,
        joel,
        mut velocity,
        _max_speed,
    ) in &mut query
    {
        let current_pos = transform.translation.xy();

        // Stay in place and track player (since we have RotateTowardsTarget component)
        velocity.0 = Vec2::ZERO;

        if timer.is_finished() {
            info!("Joel transitioning from Tracking to Retreating");

            // Remove RotateTowardsTarget component when leaving tracking state
            commands.entity(entity).remove::<RotateTowardsTarget>();

            // Set retreat target position - move back toward spawn position and beyond
            let spawn_pos = joel.spawn_position;
            let retreat_direction = (spawn_pos - current_pos).normalize_or_zero();

            // Add MoveTowardsPoint for retreating
            let retreat_target = spawn_pos + retreat_direction * 500.0;
            commands.entity(entity).insert(MoveTowardsPoint {
                target_position: retreat_target,
                stop_distance: 0.0, // Don't stop until off screen
            });

            info!("Joel set to retreat to position: {:?}", retreat_target);
        }
    }
}
