//! Collision detection systems

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{CollisionBox, Damage, Health, Player},
        events::{DamageEvent, GameOverEvent},
        resources::GameState,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            player_damage_collision,
            handle_damage_events,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

/// Check collisions between player and any entity that can deal damage
fn player_damage_collision(
    player_query: Query<(Entity, &Transform, &CollisionBox), With<Player>>,
    damage_query: Query<(Entity, &Transform, &CollisionBox), With<Damage>>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
    game_state: Res<GameState>,
) {
    if !game_state.is_game_active() {
        return;
    }

    let Ok((player_entity, player_transform, player_collision)) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.xy();
    let (player_min, player_max) = player_collision.get_rect(player_pos);

    // Check collisions with any entity that has the Damage component
    for (damage_entity, damage_transform, damage_collision) in &damage_query {
        let damage_pos = damage_transform.translation.xy();
        let (damage_min, damage_max) = damage_collision.get_rect(damage_pos);

        if rects_overlap(player_min, player_max, damage_min, damage_max) {
            damage_events.write(DamageEvent {
                target: player_entity,
                amount: 1,
            });

            commands.entity(damage_entity).try_despawn();
        }
    }
}

/// Handle damage events - apply damage and check for game over
fn handle_damage_events(
    mut damage_events: EventReader<DamageEvent>,
    mut health_query: Query<&mut Health>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    for damage_event in damage_events.read() {
        if let Ok(mut health) = health_query.get_mut(damage_event.target) {
            health.take_damage(damage_event.amount);

            info!(
                "Entity took {} damage, health: {}/{}",
                damage_event.amount, health.current, health.max
            );

            if health.is_dead() {
                game_over_events.write(GameOverEvent);
            }
        }
    }
}

/// Check if two rectangles overlap
fn rects_overlap(min1: Vec2, max1: Vec2, min2: Vec2, max2: Vec2) -> bool {
    !(max1.x < min2.x || max2.x < min1.x || max1.y < min2.y || max2.y < min1.y)
}
