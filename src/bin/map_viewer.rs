use bevy::prelude::*;
use task_masker::*;

use map::plugins::TiledCameraPlugin;
use map::tilemap::*;

fn main() {
    let mut map_viewer = App::new();
    map_viewer.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    map_viewer.add_plugins(TiledCameraPlugin);
    map_viewer.add_systems(Startup, spawn_tiled_tiles);

    map_viewer.init_state::<GameState>();
    map_viewer.insert_state::<GameState>(GameState::InGame);

    map_viewer.run();
}
