//! Joshua's Bad Week - a survival game where you play as a yellow square
//! trying to survive for 120 seconds against increasingly difficult enemies.

use bevy::prelude::*;

mod components;
mod config;
mod events;
mod resources;
mod systems;

// Re-export commonly used types
pub use resources::GameState;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        config::plugin,
        components::plugin,
        resources::plugin,
        events::plugin,
        systems::plugin,
    ));
}
