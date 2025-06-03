//! Game state management - timer, win/lose logic

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::GameEntity,
        config::GameConfig,
        events::{GameOverEvent, GameWinEvent},
        resources::{DifficultyState, GameState},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_game_timer,
            handle_game_over_events,
            handle_game_win_events,
            update_difficulty,
        )
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

/// Updates the game timer and triggers win condition
fn update_game_timer(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut game_win_events: EventWriter<GameWinEvent>,
) {
    if !game_state.is_game_active() {
        return;
    }

    game_state.time_remaining -= time.delta_secs();

    if game_state.time_remaining <= 0.0 {
        game_state.time_remaining = 0.0;
        game_state.set_game_won();
        game_win_events.write(GameWinEvent);
    }
}

/// Handle game over events
fn handle_game_over_events(
    mut events: EventReader<GameOverEvent>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    entities: Query<Entity, With<GameEntity>>,
) {
    for _ in events.read() {
        game_state.set_game_over();

        // Cleanup all game entities
        for entity in &entities {
            commands.entity(entity).try_despawn();
        }

        info!("Game Over! Player died.");
    }
}

/// Handle game win events
fn handle_game_win_events(mut events: EventReader<GameWinEvent>, game_state: Res<GameState>) {
    for _ in events.read() {
        info!(
            "Game Won! Player survived {} seconds!",
            game_state.time_remaining_seconds()
        );
    }
}

/// Update difficulty scaling over time
fn update_difficulty(
    _time: Res<Time>,
    game_state: Res<GameState>,
    mut difficulty: ResMut<DifficultyState>,
    config: Res<GameConfig>,
) {
    if !game_state.is_game_active() {
        return;
    }

    let time_elapsed = config.game_duration_seconds - game_state.time_remaining;
    difficulty.update_difficulty(time_elapsed, &config);
}
