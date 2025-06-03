//! Entity movement and physics systems

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{CollisionBox, GameEntity, OffScreenCleanup, Player},
        config::GameConfig,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (constrain_player_to_screen, cleanup_offscreen_entities)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
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
    let margin = config.spawn_distance; // Use same distance as spawn margin for consistency

    for (entity, transform, collision_box) in &query {
        let pos = transform.translation.xy();
        let size = collision_box.size;
        let max_extent = size.max_element();

        if pos.x < -screen_half_width - margin - max_extent
            || pos.x > screen_half_width + margin + max_extent
            || pos.y < -screen_half_height - margin - max_extent
            || pos.y > screen_half_height + margin + max_extent
        {
            commands.entity(entity).try_despawn();
        }
    }
}
