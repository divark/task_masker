use bevy::prelude::*;
use task_masker::*;

use std::path::PathBuf;

use map::plugins::TiledCameraPlugin;
use map::tilemap::*;

/// Renders all Tiles from some Tiled map.
pub fn spawn_tiled_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_assets: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut tilemap = Tilemap::new();
    tilemap.load_tiles_from_tiled_map(&PathBuf::from("assets/TM_map.tmx"));
    tilemap.to_isometric_coordinates();
    tilemap.y_sort_tiles();
    tilemap.flip_y_axis();
    let render_tiles = convert_tilemap_to_bevy_render_tiles(
        &tilemap,
        &asset_server.into_inner(),
        texture_atlas_assets.into_inner(),
    );

    for render_tile in render_tiles {
        commands.spawn(render_tile);
    }
}

fn main() {
    let mut map_viewer = App::new();
    map_viewer.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    map_viewer.add_plugins(TiledCameraPlugin);
    map_viewer.add_systems(Startup, spawn_tiled_tiles);

    map_viewer.init_state::<GameState>();
    map_viewer.insert_state::<GameState>(GameState::InGame);

    map_viewer.run();
}
