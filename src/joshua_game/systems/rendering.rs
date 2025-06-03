//! Basic rendering systems for game entities

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{Card, Joel, Kezia, Player},
        config::GameConfig,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera).add_systems(
        Update,
        (setup_entity_sprites, update_sprite_colors)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

#[derive(Component)]
struct EntitySprite;

#[derive(Component)]
struct GameplayCamera;

/// Setup the 2D camera for gameplay - run once on startup
fn setup_camera(mut commands: Commands) {
    commands.spawn((Name::new("Gameplay Camera"), Camera2d, GameplayCamera));
}

/// Add sprite components to entities that need visual representation
fn setup_entity_sprites(
    mut commands: Commands,
    config: Res<GameConfig>,
    // Query for entities without sprites
    player_query: Query<Entity, (With<Player>, Without<EntitySprite>)>,
    kezia_query: Query<Entity, (With<Kezia>, Without<EntitySprite>)>,
    joel_query: Query<Entity, (With<Joel>, Without<EntitySprite>)>,
    card_query: Query<Entity, (With<Card>, Without<EntitySprite>)>,
) {
    // Setup player sprite
    for entity in &player_query {
        commands.entity(entity).insert((
            Sprite {
                color: config.player_color,
                custom_size: Some(Vec2::splat(config.player_size)),
                ..default()
            },
            EntitySprite,
        ));
    }

    // Setup Kezia sprites
    for entity in &kezia_query {
        commands.entity(entity).insert((
            Sprite {
                color: config.kezia_color,
                custom_size: Some(Vec2::new(config.kezia_width, config.kezia_height)),
                ..default()
            },
            EntitySprite,
        ));
    }

    // Setup Joel sprites
    for entity in &joel_query {
        commands.entity(entity).insert((
            Sprite {
                color: config.joel_color,
                custom_size: Some(Vec2::new(config.joel_width, config.joel_height)),
                ..default()
            },
            EntitySprite,
        ));
    }

    // Setup card sprites
    for entity in &card_query {
        commands.entity(entity).insert((
            Sprite {
                color: config.card_color,
                custom_size: Some(Vec2::new(config.card_width, config.card_height)),
                ..default()
            },
            EntitySprite,
        ));
    }
}

/// Update sprite colors based on game state (health, charging, etc.)
fn update_sprite_colors(
    config: Res<GameConfig>,
    mut player_query: Query<
        (&mut Sprite, &crate::joshua_game::components::Health),
        (With<Player>, With<EntitySprite>),
    >,
    mut joel_query: Query<(&mut Sprite, &Joel), (With<Joel>, With<EntitySprite>, Without<Player>)>,
) {
    // Update player color based on health
    for (mut sprite, health) in &mut player_query {
        let health_ratio = health.health_ratio();
        if health_ratio <= 0.3 {
            sprite.color = config.player_low_health_color;
        } else {
            sprite.color = config.player_color;
        }
    }

    // Update Joel color when charging
    for (mut sprite, joel) in &mut joel_query {
        if joel.is_charging {
            sprite.color = config.joel_charging_color;
        } else {
            sprite.color = config.joel_color;
        }
    }
}
