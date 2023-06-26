use bevy::prelude::*;

use crate::GameState;
use super::screens::*;

#[derive(Default)]
pub struct StartupScreenPlugin;

impl Plugin for StartupScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_start_screen.in_schedule(OnEnter(GameState::Start)));
        app.add_system(despawn_start_screen.in_schedule(OnExit(GameState::Start)));
        app.add_system(spawn_end_screen.in_schedule(OnEnter(GameState::End)));
        app.add_system(despawn_end_screen.in_schedule(OnExit(GameState::End)));
        app.add_system(cycle_screens);
    }
}
