use bevy::prelude::*;

use super::screens::spawn_start_screen;

#[derive(Default)]
pub struct StartupScreenPlugin;

impl Plugin for StartupScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_start_screen);
    }
}