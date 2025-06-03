//! Entity movement and physics systems

use bevy::prelude::*;

use crate::{
    joshua_game::{
        components::{Player, Movement, CollisionBox, OffScreenCleanup, GameEntity},
        config::GameConfig,
        resources::{GameState},
    },
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_movement,
            constrain_player_to_screen,
            cleanup_offscreen_entities,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

/// Apply velocity to entity positions
fn apply_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Movement)>,
    game_state: Res<GameState>,
) {
    if !game_state.is_game_active() {
        return;
    }

    for (mut transform, movement) in &mut query {
        transform.translation += movement.velocity.extend(0.0) * time.delta_secs();
        transform.rotation = Quat::from_rotation_z(movement.rotation);
    }
}

/// Keep the player within screen boundaries
fn constrain_player_to_screen(
    mut player_query: Query<(&mut Transform, &CollisionBox), With<Player>>,
    config: Res<GameConfig>,
) {
    let Ok((mut transform, collision_box)) = player_query.single_mut() else {
        return;
    };

    let half_size = collision_box.size / 2.0;
    let screen_half_width = config.screen_width / 2.0;
    let screen_half_height = config.screen_height / 2.0;

    // Constrain X position (note: Bevy uses center-origin coordinates)
    if transform.translation.x - half_size.x < -screen_half_width {
        transform.translation.x = -screen_half_width + half_size.x;
    } else if transform.translation.x + half_size.x > screen_half_width {
        transform.translation.x = screen_half_width - half_size.x;
    }

    // Constrain Y position
    if transform.translation.y - half_size.y < -screen_half_height {
        transform.translation.y = -screen_half_height + half_size.y;
    } else if transform.translation.y + half_size.y > screen_half_height {
        transform.translation.y = screen_half_height - half_size.y;
    }
}

/// Remove entities that have moved off screen
fn cleanup_offscreen_entities(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &CollisionBox), (With<OffScreenCleanup>, With<GameEntity>)>,
    config: Res<GameConfig>,
) {
    let screen_half_width = config.screen_width / 2.0;
    let screen_half_height = config.screen_height / 2.0;
    let margin = 100.0; // Extra margin before cleanup

    for (entity, transform, collision_box) in &query {
        let pos = transform.translation.xy();
        let size = collision_box.size;
        let max_extent = size.max_element();

        if pos.x < -screen_half_width - margin - max_extent
            || pos.x > screen_half_width + margin + max_extent
            || pos.y < -screen_half_height - margin - max_extent
            || pos.y > screen_half_height + margin + max_extent
        {
            commands.entity(entity).despawn();
        }
    }
} 