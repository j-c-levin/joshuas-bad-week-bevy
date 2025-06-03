//! UI systems for displaying game information

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    joshua_game::{
        components::{Health, Player},
        config::GameConfig,
        resources::GameState,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            setup_ui,
            update_health_display,
            update_timer_display,
            update_game_over_display,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct GameUI;

/// Setup UI elements when gameplay starts
fn setup_ui(mut commands: Commands, ui_query: Query<&GameUI>, config: Res<GameConfig>) {
    // Only setup UI once
    if !ui_query.is_empty() {
        return;
    }

    // Create UI root
    commands
        .spawn((
            Name::new("Game UI"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            GameUI,
        ))
        .with_children(|parent| {
            // Top section for health and timer
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // Health display
                    parent.spawn((
                        Text::new("Health: 10/10"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(config.ui_text_color),
                        HealthText,
                    ));

                    // Timer display
                    parent.spawn((
                        Text::new("Time: 120"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(config.ui_text_color),
                        TimerText,
                    ));
                });

            // Center section for game over message
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(config.game_over_color),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    ..default()
                },
                GameOverText,
            ));
        });
}

/// Update health display
fn update_health_display(
    player_query: Query<&Health, With<Player>>,
    mut health_text_query: Query<&mut Text, With<HealthText>>,
) {
    let Ok(health) = player_query.single() else {
        return;
    };

    let Ok(mut text) = health_text_query.single_mut() else {
        return;
    };

    **text = format!("Health: {}/{}", health.current, health.max);
}

/// Update timer display
fn update_timer_display(
    game_state: Res<GameState>,
    mut timer_text_query: Query<&mut Text, With<TimerText>>,
) {
    let Ok(mut text) = timer_text_query.single_mut() else {
        return;
    };

    **text = format!("Time: {}", game_state.time_remaining_seconds());
}

/// Update game over display
fn update_game_over_display(
    game_state: Res<GameState>,
    mut game_over_text_query: Query<&mut Text, With<GameOverText>>,
) {
    let Ok(mut text) = game_over_text_query.single_mut() else {
        return;
    };

    if game_state.is_game_won() {
        **text = "YOU WIN!".to_string();
    } else if game_state.is_game_over() {
        **text = "GAME OVER".to_string();
    } else {
        **text = "".to_string();
    }
}
