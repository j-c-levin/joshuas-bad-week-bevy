//! Collision detection systems

use bevy::prelude::*;

use crate::{
    joshua_game::{
        components::{Player, Kezia, Joel, Card, CollisionBox, Health},
        events::{DamageEvent, DamageSource, GameOverEvent},
        config::GameConfig,
        resources::GameState,
    },
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            player_enemy_collision,
            player_card_collision,
            handle_damage_events,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

/// Check collisions between player and enemies
fn player_enemy_collision(
    player_query: Query<(Entity, &Transform, &CollisionBox), With<Player>>,
    kezia_query: Query<(Entity, &Transform, &CollisionBox), (With<Kezia>, Without<Player>)>,
    joel_query: Query<(Entity, &Transform, &CollisionBox), (With<Joel>, Without<Player>, Without<Kezia>)>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
    config: Res<GameConfig>,
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

    // Check Kezia collisions
    for (kezia_entity, kezia_transform, kezia_collision) in &kezia_query {
        let kezia_pos = kezia_transform.translation.xy();
        let (kezia_min, kezia_max) = kezia_collision.get_rect(kezia_pos);

        if rects_overlap(player_min, player_max, kezia_min, kezia_max) {
            damage_events.write(DamageEvent {
                target: player_entity,
                amount: config.kezia_damage,
                position: player_pos,
                source: DamageSource::Kezia,
            });

            // Destroy the Kezia that hit the player
            commands.entity(kezia_entity).despawn();
        }
    }

    // Check Joel collisions
    for (_joel_entity, joel_transform, joel_collision) in &joel_query {
        let joel_pos = joel_transform.translation.xy();
        let (joel_min, joel_max) = joel_collision.get_rect(joel_pos);

        if rects_overlap(player_min, player_max, joel_min, joel_max) {
            damage_events.write(DamageEvent {
                target: player_entity,
                amount: config.joel_damage,
                position: player_pos,
                source: DamageSource::Joel,
            });

            // Joel doesn't get destroyed on collision
        }
    }
}

/// Check collisions between player and cards
fn player_card_collision(
    player_query: Query<(Entity, &Transform, &CollisionBox), With<Player>>,
    card_query: Query<(Entity, &Transform, &CollisionBox, &Card), Without<Player>>,
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

    for (card_entity, card_transform, card_collision, card) in &card_query {
        let card_pos = card_transform.translation.xy();
        let (card_min, card_max) = card_collision.get_rect(card_pos);

        if rects_overlap(player_min, player_max, card_min, card_max) {
            damage_events.write(DamageEvent {
                target: player_entity,
                amount: card.damage,
                position: player_pos,
                source: DamageSource::Card,
            });

            // Destroy the card that hit the player
            commands.entity(card_entity).despawn();
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
            
            info!("Entity took {} damage, health: {}/{}", 
                  damage_event.amount, health.current, health.max);

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