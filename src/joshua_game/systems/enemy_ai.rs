//! Enemy AI systems - Kezia tracking and Joel state machine

use bevy::prelude::*;

use crate::{
    joshua_game::{
        components::{Player, Kezia, Joel, Movement, JoelState},
        config::GameConfig,
        resources::GameState,
        events::CardSpawnEvent,
    },
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            kezia_ai_system,
            joel_ai_system,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

/// Kezia AI: Track player for duration, then continue straight
fn kezia_ai_system(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Kezia>)>,
    mut kezia_query: Query<(&mut Kezia, &mut Movement, &Transform), (With<Kezia>, Without<Player>)>,
    config: Res<GameConfig>,
    game_state: Res<GameState>,
) {
    if !game_state.is_game_active() {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.xy();
    let delta = time.delta_secs();

    for (mut kezia, mut movement, transform) in &mut kezia_query {
        kezia.tracking_timer += delta;

        // Check if tracking duration is over
        if kezia.tracking_timer >= config.kezia_tracking_duration {
            kezia.is_tracking = false;
        }

        if kezia.is_tracking {
            // Calculate desired direction to player
            let kezia_pos = transform.translation.xy();
            let direction_to_player = (player_pos - kezia_pos).normalize_or_zero();
            
            if direction_to_player != Vec2::ZERO {
                let target_rotation = direction_to_player.y.atan2(direction_to_player.x);
                
                // Smoothly rotate towards target
                let rotation_diff = (target_rotation - movement.rotation + std::f32::consts::PI) 
                    % (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
                let max_rotation_change = config.kezia_turn_rate * delta;
                
                if rotation_diff.abs() <= max_rotation_change {
                    movement.rotation = target_rotation;
                } else {
                    movement.rotation += rotation_diff.signum() * max_rotation_change;
                }
            }
        }
        
        // Update velocity based on current rotation
        movement.velocity = Vec2::new(
            movement.rotation.cos() * config.kezia_speed,
            movement.rotation.sin() * config.kezia_speed,
        );
    }
}

/// Joel AI: Three-state machine (Approaching, Tracking, Retreating)
fn joel_ai_system(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Joel>)>,
    mut joel_query: Query<(&mut Joel, &mut Movement, &Transform), (With<Joel>, Without<Player>)>,
    config: Res<GameConfig>,
    game_state: Res<GameState>,
    mut card_events: EventWriter<CardSpawnEvent>,
) {
    if !game_state.is_game_active() {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.xy();
    let delta = time.delta_secs();

    for (mut joel, mut movement, transform) in &mut joel_query {
        joel.state_timer += delta;
        let joel_pos = transform.translation.xy();

        match joel.state {
            JoelState::Approaching => {
                // Move toward target position
                let direction = (joel.target_position - joel_pos).normalize_or_zero();
                let distance = joel_pos.distance(joel.target_position);
                
                if distance <= 5.0 {
                    // Close enough, switch to tracking
                    joel.state = JoelState::Tracking;
                    joel.state_timer = 0.0;
                    movement.velocity = Vec2::ZERO;
                } else {
                    movement.velocity = direction * config.joel_speed;
                }
            },
            
            JoelState::Tracking => {
                // Stay in place and track player
                movement.velocity = Vec2::ZERO;
                
                // Rotate to face player
                let direction_to_player = (player_pos - joel_pos).normalize_or_zero();
                if direction_to_player != Vec2::ZERO {
                    let target_rotation = direction_to_player.y.atan2(direction_to_player.x) + std::f32::consts::PI / 2.0;
                    
                    let rotation_diff = (target_rotation - movement.rotation + std::f32::consts::PI) 
                        % (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
                    let max_rotation_change = config.joel_turn_rate * delta;
                    
                    if rotation_diff.abs() <= max_rotation_change {
                        movement.rotation = target_rotation;
                    } else {
                        movement.rotation += rotation_diff.signum() * max_rotation_change;
                    }
                }
                
                // Handle card firing
                joel.card_fire_timer += delta;
                
                // Start charging before firing
                if joel.card_fire_timer >= config.joel_card_fire_rate - 0.5 && !joel.is_charging {
                    joel.is_charging = true;
                    joel.charge_timer = 0.0;
                }
                
                if joel.is_charging {
                    joel.charge_timer += delta;
                }
                
                // Fire card
                if joel.card_fire_timer >= config.joel_card_fire_rate {
                    let direction_to_player = (player_pos - joel_pos).normalize_or_zero();
                    card_events.write(CardSpawnEvent {
                        position: joel_pos,
                        direction: direction_to_player,
                        source: Entity::PLACEHOLDER, // TODO: Pass actual entity
                    });
                    
                    joel.card_fire_timer = 0.0;
                    joel.is_charging = false;
                    joel.charge_timer = 0.0;
                }
                
                // Check if tracking duration is over
                if joel.state_timer >= config.joel_tracking_duration {
                    joel.state = JoelState::Retreating;
                    joel.state_timer = 0.0;
                }
            },
            
            JoelState::Retreating => {
                // Move back toward spawn position and beyond
                let direction = (joel.spawn_position - joel_pos).normalize_or_zero();
                movement.velocity = direction * config.joel_speed;
            },
        }
    }
} 