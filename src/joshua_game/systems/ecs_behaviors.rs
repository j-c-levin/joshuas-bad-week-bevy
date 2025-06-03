//! Modular ECS behavior systems for entity behaviors

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{
            Joel, KeziaState, LifetimeTimer, MaxSpeed, MoveInDirection, MoveTowardsPoint,
            NewJoelState, PlayerTarget, ProjectileLauncher, RotateTowardsTarget, SpawnSide, TargetPlayer, Timer,
            TurnRate, Velocity,
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
            track_velocity_system,
            move_towards_point_system,
            kezia_state_system,
            joel_state_system,
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
            lifetime_timer_system,
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
        // If velocity is zero or very small, set it based on current rotation
        if velocity.0.length() < 0.1 {
            let rotation = transform.rotation.to_euler(EulerRot::ZYX).0;
            velocity.0 = Vec2::new(rotation.cos(), rotation.sin()) * max_speed.0;
        }
    }
}

// ==================== Behavior Systems ====================

/// Rotate entities towards their target (only during tracking state for Kezia)
fn rotate_towards_target_system(
    time: Res<Time>,
    target_query: Query<&Transform, With<PlayerTarget>>,
    mut kezia_rotate_query: Query<
        (&TargetPlayer, &RotateTowardsTarget, &mut Transform, &TurnRate, &KeziaState),
        (
            Without<PlayerTarget>,
            With<KeziaState>,
            Without<NewJoelState>,
        ),
    >,
    mut joel_rotate_query: Query<
        (
            &TargetPlayer,
            &RotateTowardsTarget,
            &mut Transform,
            &TurnRate,
            &NewJoelState,
        ),
        (
            Without<PlayerTarget>,
            With<NewJoelState>,
            Without<KeziaState>,
        ),
    >,
) {
    let delta = time.delta_secs();

    // Handle Kezia rotation
    for (target_player, rotate_target, mut transform, turn_rate, kezia_state) in &mut kezia_rotate_query {
        match *kezia_state {
            KeziaState::Tracking => {
                // Rotate towards player during tracking
                if let Ok(target_transform) = target_query.get(target_player.target_entity) {
                    let direction =
                        target_transform.translation.xy() - transform.translation.xy();
                    if direction.length() > 0.1 {
                        let target_rotation =
                            direction.y.atan2(direction.x) + rotate_target.offset_angle;
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
            KeziaState::MovingStraight => {
                // During MovingStraight, rotation should match velocity direction
                // This is handled by the velocity update in kezia_state_system
                // No additional rotation needed here
            }
        }
    }

    // Handle Joel rotation
    for (target_player, rotate_target, mut transform, turn_rate, joel_state) in &mut joel_rotate_query {
        match *joel_state {
            NewJoelState::Tracking => {
                // Rotate towards player during tracking
                if let Ok(target_transform) = target_query.get(target_player.target_entity) {
                    let direction =
                        target_transform.translation.xy() - transform.translation.xy();
                    if direction.length() > 0.1 {
                        let target_rotation =
                            direction.y.atan2(direction.x) + rotate_target.offset_angle;
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
            NewJoelState::Entering | NewJoelState::Approaching | NewJoelState::Retreating => {
                // No rotation during entering, approaching, or retreating
            }
        }
    }
}

/// Move entities towards a specific point
fn move_towards_point_system(
    mut query: Query<(&mut MoveTowardsPoint, &Transform, &mut Velocity, &MaxSpeed), Without<Joel>>,
) {
    for (move_towards, transform, mut velocity, max_speed) in &mut query {
        let current_pos = transform.translation.xy();
        let distance = current_pos.distance(move_towards.target_position);

        if distance <= move_towards.stop_distance {
            velocity.0 = Vec2::ZERO;
        } else {
            let direction = (move_towards.target_position - current_pos).normalize_or_zero();
            velocity.0 = direction * max_speed.0;
        }
    }
}

/// Handle projectile launching
fn projectile_launcher_system(
    time: Res<Time>,
    target_query: Query<&Transform, With<PlayerTarget>>,
    mut launcher_query: Query<(&TargetPlayer, &mut ProjectileLauncher, &Transform), Without<PlayerTarget>>,
    mut spawn_events: EventWriter<CardSpawnEvent>,
) {
    let delta = time.delta_secs();

    for (target_player, mut launcher, transform) in &mut launcher_query {
        launcher.timer += delta;

        if launcher.timer >= launcher.fire_rate {
            if let Ok(target_transform) = target_query.get(target_player.target_entity) {
                let direction = (target_transform.translation.xy()
                    - transform.translation.xy())
                .normalize_or_zero();

                spawn_events.write(CardSpawnEvent {
                    position: transform.translation.xy(),
                    direction,
                });

                launcher.timer = 0.0;
            }
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

/// Handle lifetime timers and despawn expired entities
fn lifetime_timer_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut LifetimeTimer)>,
) {
    let delta = time.delta_secs();

    for (entity, mut lifetime) in &mut query {
        lifetime.remaining -= delta;

        if lifetime.remaining <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

// ==================== State-Specific Systems ====================

/// Handle Kezia state transitions
fn kezia_state_system(
    mut query: Query<(
        &mut KeziaState,
        &mut Timer,
        &mut Velocity,
        &mut Transform,
        &MaxSpeed,
    )>,
) {
    for (mut state, timer, mut velocity, mut transform, max_speed) in &mut query {
        match *state {
            KeziaState::Tracking => {
                // During tracking, ensure velocity matches rotation direction (forward movement)
                let rotation = transform.rotation.to_euler(EulerRot::ZYX).0;
                velocity.0 = Vec2::new(rotation.cos(), rotation.sin()) * max_speed.0;

                if timer.is_finished() {
                    *state = KeziaState::MovingStraight;

                    // Set velocity to continue in current direction (which is already correct from above)
                    // Update rotation to match velocity direction for consistent movement
                    let velocity_angle = velocity.0.y.atan2(velocity.0.x);
                    transform.rotation = Quat::from_rotation_z(velocity_angle);
                }
            }
            KeziaState::MovingStraight => {
                // Continue moving straight - velocity is maintained by move_in_direction_system
                // Ensure rotation stays aligned with movement direction
                if velocity.0.length() > 0.1 {
                    let velocity_angle = velocity.0.y.atan2(velocity.0.x);
                    transform.rotation = Quat::from_rotation_z(velocity_angle);
                }
            }
        }
    }
}

/// Handle Joel state transitions
fn joel_state_system(
    mut query: Query<(
        &mut NewJoelState,
        &mut MoveTowardsPoint,
        &mut ProjectileLauncher,
        &mut Timer,
        &Transform,
        &Joel, // Added Joel component to access spawn position
        &mut Velocity,
        &MaxSpeed,
    )>,
    config: Res<GameConfig>,
) {
    let screen_half_width = config.screen_width / 2.0;
    let screen_half_height = config.screen_height / 2.0;

    for (
        mut state,
        mut move_towards,
        _launcher,
        mut timer,
        transform,
        joel,
        mut velocity,
        max_speed,
    ) in &mut query
    {
        let current_pos = transform.translation.xy();

        match *state {
            NewJoelState::Entering => {
                // Check if Joel has entered the screen area
                let has_entered_screen = match joel.spawn_side {
                    SpawnSide::Top => current_pos.y <= screen_half_height,
                    SpawnSide::Right => current_pos.x <= screen_half_width,
                    SpawnSide::Bottom => current_pos.y >= -screen_half_height,
                    SpawnSide::Left => current_pos.x >= -screen_half_width,
                };

                if has_entered_screen {
                    info!("Joel entered screen, transitioning from Entering to Approaching");
                    *state = NewJoelState::Approaching;

                    // Set velocity to move toward target position
                    let direction = (joel.target_position - current_pos).normalize_or_zero();
                    velocity.0 = direction * max_speed.0;
                }
                // During Entering state, continue moving perpendicular to spawn side
                // Velocity was set during spawn and should continue
            }
            NewJoelState::Approaching => {
                let distance = current_pos.distance(move_towards.target_position);
                if distance <= move_towards.stop_distance {
                    info!("Joel transitioning from Approaching to Tracking");
                    *state = NewJoelState::Tracking;
                    timer.reset();
                    velocity.0 = Vec2::ZERO; // Stop moving
                } else {
                    // Continue moving toward target
                    let direction = (joel.target_position - current_pos).normalize_or_zero();
                    velocity.0 = direction * max_speed.0;
                }
            }
            NewJoelState::Tracking => {
                // Stay in place and track player
                velocity.0 = Vec2::ZERO;

                if timer.is_finished() {
                    info!("Joel transitioning from Tracking to Retreating");
                    *state = NewJoelState::Retreating;

                    // Set retreat target position - move back toward spawn position and beyond
                    let spawn_pos = joel.spawn_position;
                    let retreat_direction = (spawn_pos - current_pos).normalize_or_zero();

                    // Set target position far off screen in retreat direction
                    move_towards.target_position = spawn_pos + retreat_direction * 500.0;
                    move_towards.stop_distance = 0.0; // Don't stop until off screen

                    // Set retreat velocity
                    velocity.0 = retreat_direction * max_speed.0;

                    info!(
                        "Joel set to retreat to position: {:?}",
                        move_towards.target_position
                    );
                }
            }
            NewJoelState::Retreating => {
                // Continue retreating - velocity should maintain retreat direction
                let retreat_direction = (joel.spawn_position - current_pos).normalize_or_zero();
                velocity.0 = retreat_direction * max_speed.0;
            }
        }
    }
}

// ==================== Helper Systems ====================

/// Update velocity to track target entities
fn track_velocity_system(
    target_query: Query<&Transform, With<PlayerTarget>>,
    mut tracker_query: Query<
        (
            &TargetPlayer,
            &Transform,
            &mut Velocity,
            &MaxSpeed,
            &TurnRate,
        ),
        (Without<PlayerTarget>, Without<KeziaState>), // Exclude Kezia entities
    >,
) {
    for (target_player, transform, mut velocity, max_speed, _turn_rate) in &mut tracker_query {
        if let Ok(target_transform) = target_query.get(target_player.target_entity) {
            let direction = (target_transform.translation.xy() - transform.translation.xy())
                .normalize_or_zero();
            velocity.0 = direction * max_speed.0;
        }
    }
}
