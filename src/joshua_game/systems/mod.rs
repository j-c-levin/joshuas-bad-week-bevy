//! Game systems for Joshua's Bad Week

use bevy::prelude::*;

mod collision;
mod ecs_behaviors;
mod enemy_ai;
mod game_state;
mod input;
mod movement;
mod rendering;
mod spawning;
mod ui;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        input::plugin,
        movement::plugin,
        enemy_ai::plugin,
        ecs_behaviors::plugin,
        collision::plugin,
        spawning::plugin,
        game_state::plugin,
        rendering::plugin,
        ui::plugin,
    ));
}
