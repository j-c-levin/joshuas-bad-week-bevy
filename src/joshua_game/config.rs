//! Centralized configuration for all game settings - ported from MonoGame GameConfig.cs

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameConfig>();
}

/// Centralized configuration for all game settings
#[derive(Resource)]
pub struct GameConfig {
    // Screen Settings
    pub screen_width: f32,
    pub screen_height: f32,

    // Player Settings
    pub player_size: f32,
    pub player_speed: f32,
    pub player_color: Color,

    // Game Settings
    pub game_duration_seconds: f32,
    pub initial_health: i32,

    // UI Settings
    pub ui_text_color: Color,

    // Effect Colors
    pub player_low_health_color: Color,
    pub joel_charging_color: Color,

    // Input Settings
    pub diagonal_movement_normalizer: f32,

    // Kezia Enemy Settings
    pub kezia_width: f32,
    pub kezia_height: f32,
    pub kezia_speed: f32,
    pub kezia_color: Color,
    pub kezia_tracking_duration: f32,
    pub kezia_turn_rate: f32,
    pub kezia_damage: i32,
    pub kezia_spawn_distance: f32,

    // Kezia Spawning Settings
    pub initial_spawn_interval: f32,
    pub min_spawn_interval: f32,
    pub difficulty_ramp_duration: f32,

    // Joel Enemy Settings
    pub joel_width: f32,
    pub joel_height: f32,
    pub joel_speed: f32,
    pub joel_color: Color,
    pub joel_approach_distance: f32,
    pub joel_tracking_duration: f32,
    pub joel_turn_rate: f32,
    pub joel_card_fire_rate: f32,
    pub joel_damage: i32,

    // Card Projectile Settings
    pub card_width: f32,
    pub card_height: f32,
    pub card_speed: f32,
    pub card_color: Color,
    pub card_damage: i32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            // Screen Settings
            screen_width: 800.0,
            screen_height: 600.0,

            // Player Settings
            player_size: 20.0,
            player_speed: 200.0,
            player_color: Color::srgb(1.0, 1.0, 0.0), // Yellow

            // Game Settings
            game_duration_seconds: 120.0,
            initial_health: 10,

            // UI Settings
            ui_text_color: Color::WHITE,

            // Effect Colors
            player_low_health_color: Color::srgb(1.0, 0.0, 0.0), // Red
            joel_charging_color: Color::WHITE,

            // Input Settings
            diagonal_movement_normalizer: 0.707, // 1/sqrt(2)

            // Kezia Enemy Settings
            kezia_width: 20.0,
            kezia_height: 10.0,
            kezia_speed: 120.0,
            kezia_color: Color::srgb(1.0, 0.0, 0.0), // Red
            kezia_tracking_duration: 5.0,
            kezia_turn_rate: 2.0,
            kezia_damage: 1,
            kezia_spawn_distance: 50.0,

            // Kezia Spawning Settings
            initial_spawn_interval: 2.0,
            min_spawn_interval: 0.3,
            difficulty_ramp_duration: 90.0,

            // Joel Enemy Settings
            joel_width: 30.0,
            joel_height: 15.0,
            joel_speed: 100.0,
            joel_color: Color::srgb(0.0, 1.0, 1.0), // Cyan
            joel_approach_distance: 50.0,
            joel_tracking_duration: 8.0,
            joel_turn_rate: 5.0,
            joel_card_fire_rate: 1.5,
            joel_damage: 1,

            // Card Projectile Settings
            card_width: 6.0,
            card_height: 4.0,
            card_speed: 150.0,
            card_color: Color::srgb(0.5, 1.0, 0.5), // Lime green
            card_damage: 1,
        }
    }
}
