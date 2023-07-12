use bevy::prelude::*;

use super::screens::*;
use crate::GameState;

#[derive(Default)]
pub struct StartupScreenPlugin;

impl Plugin for StartupScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_start_screen.in_schedule(OnEnter(GameState::Start)));
        app.add_system(despawn_start_screen.in_schedule(OnExit(GameState::Start)));

        app.add_system(spawn_ingame_screen.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(insert_counting_information.in_set(OnUpdate(GameState::InGame)));
        app.add_system(decrement_health_timer.in_set(OnUpdate(GameState::InGame)));
        app.add_system(update_healthbar_progress.in_set(OnUpdate(GameState::InGame)));
        app.add_system(end_ingame_on_no_health.in_set(OnUpdate(GameState::InGame)));
        app.add_system(despawn_ingame_screen.in_schedule(OnExit(GameState::InGame)));

        app.add_system(spawn_end_screen.in_schedule(OnEnter(GameState::End)));
        app.add_system(despawn_end_screen.in_schedule(OnExit(GameState::End)));
        app.add_system(cycle_screens);
    }
}
