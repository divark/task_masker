use bevy::prelude::*;

use super::screens::*;
use crate::GameState;

#[derive(Default)]
pub struct StartupScreenPlugin;

impl Plugin for StartupScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Start), spawn_start_screen);
        app.add_systems(OnExit(GameState::Start), despawn_start_screen);

        app.add_systems(OnEnter(GameState::InGame), spawn_ingame_screen);

        app.add_systems(
            Update,
            (
                insert_counting_information,
                decrement_health_timer,
                update_healthbar_progress,
                end_ingame_on_no_health,
            )
                .run_if(in_state(GameState::InGame)),
        );

        app.add_systems(OnExit(GameState::InGame), despawn_ingame_screen);

        app.add_systems(OnEnter(GameState::End), spawn_end_screen);
        app.add_systems(OnExit(GameState::End), despawn_end_screen);
        app.add_systems(Update, cycle_screens);
    }
}
