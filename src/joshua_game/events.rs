//! Game events for Joshua's Bad Week

use bevy::prelude::*;

use crate::joshua_game::components::SpawnSide;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<DamageEvent>()
        .add_event::<EnemySpawnEvent>()
        .add_event::<CardSpawnEvent>()
        .add_event::<GameOverEvent>()
        .add_event::<GameWinEvent>()
        .add_event::<EnemyDeathEvent>();
}

/// Event fired when an entity takes damage
#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: i32,
}

/// Event fired when an enemy should be spawned
#[derive(Event)]
pub struct EnemySpawnEvent {
    pub enemy_type: EnemyType,
    pub position: Vec2,
    pub spawn_side: SpawnSide,
}

#[derive(Clone, Copy)]
pub enum EnemyType {
    Kezia,
    Joel,
}

/// Event fired when Joel should fire a card
#[derive(Event)]
pub struct CardSpawnEvent {
    pub position: Vec2,
    pub direction: Vec2,
}

/// Event fired when the player dies
#[derive(Event)]
pub struct GameOverEvent;

/// Event fired when the player survives 120 seconds
#[derive(Event)]
pub struct GameWinEvent;

/// Event fired when an enemy dies/is destroyed
#[derive(Event)]
pub struct EnemyDeathEvent {
}
