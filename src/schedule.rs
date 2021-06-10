//! Schedule builders which execute systems based on TurnState.

use crate::prelude::*;

pub fn build_input_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(input_system())
        .flush()
        .add_system(map_render_system())
        .add_system(entity_render_system())
        .add_system(ui_render_system())
        .build()
}

pub fn build_player_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(combat_system())
        .flush()
        .add_system(movement_system())
        .flush()
        .add_system(mining_system())
        .flush()
        .add_system(map_render_system())
        .add_system(entity_render_system())
        .add_system(ui_render_system())
        .add_system(cycle_turn_system())
        .build()
}

pub fn build_enemy_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(random_movement_system())
        .add_system(targeted_movement_system())
        .flush()
        .add_system(movement_system())
        .flush()
        .add_system(mining_system())
        .flush()
        .add_system(map_render_system())
        .add_system(entity_render_system())
        .add_system(ui_render_system())
        .add_system(cycle_turn_system())
        .build()
}
