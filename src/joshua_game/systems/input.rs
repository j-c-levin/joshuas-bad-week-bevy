//! Player input handling system

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{MaxSpeed, Player, PlayerTarget, Velocity},
        config::GameConfig,
        resources::GameState,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        player_input_system
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

/// Player input system using new ECS components
fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (&mut Velocity, &mut Transform, &MaxSpeed),
        (With<Player>, With<PlayerTarget>),
    >,
    config: Res<GameConfig>,
    game_state: Res<GameState>,
) {
    // Only handle input if the game is active
    if !game_state.is_game_active() {
        return;
    }

    let Ok((mut velocity, mut transform, max_speed)) = player_query.single_mut() else {
        return;
    };

    let mut input_direction = Vec2::ZERO;

    // Handle arrow keys and WASD
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        input_direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        input_direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        input_direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        input_direction.x += 1.0;
    }

    if input_direction != Vec2::ZERO {
        // Normalize diagonal movement to prevent faster diagonal speed
        if input_direction.x != 0.0 && input_direction.y != 0.0 {
            input_direction *= config.diagonal_movement_normalizer;
        }

        // Calculate rotation to face movement direction
        let rotation = input_direction.y.atan2(input_direction.x);
        transform.rotation = Quat::from_rotation_z(rotation);

        // Update velocity
        velocity.0 = input_direction * max_speed.0;
    } else {
        velocity.0 = Vec2::ZERO;
    }
}
