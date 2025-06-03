//! Global game state resources

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameState>()
        .init_resource::<DifficultyState>();
}

/// Main game state resource
#[derive(Resource)]
pub struct GameState {
    pub time_remaining: f32,
    pub state: GameStateEnum,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            time_remaining: 120.0, // 120 seconds to survive
            state: GameStateEnum::Playing,
        }
    }
}

impl GameState {
    pub fn time_remaining_seconds(&self) -> i32 {
        self.time_remaining.ceil() as i32
    }

    pub fn is_game_active(&self) -> bool {
        matches!(self.state, GameStateEnum::Playing)
    }

    pub fn is_game_won(&self) -> bool {
        matches!(self.state, GameStateEnum::Won)
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self.state, GameStateEnum::GameOver)
    }

    pub fn set_game_over(&mut self) {
        if self.state == GameStateEnum::Playing {
            self.state = GameStateEnum::GameOver;
        }
    }

    pub fn set_game_won(&mut self) {
        if self.state == GameStateEnum::Playing {
            self.state = GameStateEnum::Won;
        }
    }

    pub fn reset(&mut self) {
        self.time_remaining = 120.0;
        self.state = GameStateEnum::Playing;
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum GameStateEnum {
    Playing,
    Won,
    GameOver,
    Paused,
}

/// Tracks enemy spawning difficulty over time
#[derive(Resource)]
pub struct DifficultyState {
    pub kezia_spawn_timer: f32,
    pub joel_spawn_timer: f32,
    pub current_kezia_interval: f32,
    pub current_joel_interval: f32,
}

impl Default for DifficultyState {
    fn default() -> Self {
        Self {
            kezia_spawn_timer: 0.0,
            joel_spawn_timer: 0.0,
            current_kezia_interval: 2.0, // Start with 2 second intervals
            current_joel_interval: 8.0,  // Joel spawns less frequently
        }
    }
}

impl DifficultyState {
    pub fn update_difficulty(
        &mut self,
        time_elapsed: f32,
        config: &crate::joshua_game::config::GameConfig,
    ) {
        // Calculate difficulty progression (0.0 to 1.0 over 90 seconds)
        let difficulty_progress = (time_elapsed / config.difficulty_ramp_duration).min(1.0);

        // Interpolate between initial and minimum spawn intervals
        self.current_kezia_interval = config.initial_spawn_interval
            - (config.initial_spawn_interval - config.min_spawn_interval) * difficulty_progress;

        // Joel spawning gets more frequent too, but less aggressively
        self.current_joel_interval = 8.0 - 5.0 * difficulty_progress;
        self.current_joel_interval = self.current_joel_interval.max(2.0);
    }
}
