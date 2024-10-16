use bevy::prelude::*;

use super::path_finding::*;
use crate::map::camera::*;
use crate::map::tilemap::*;
use crate::GameState;

#[derive(Event)]
pub struct TilePosEvent {
    pub destination: TileGridCoordinates,
}

impl TilePosEvent {
    pub fn new(destination: TileGridCoordinates) -> Self {
        TilePosEvent { destination }
    }
}

#[derive(Default)]
pub struct PathFindingPlugin;

impl Plugin for PathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TilePosEvent>().add_systems(
            Update,
            (
                create_ground_graph,
                create_air_graph,
                create_water_graph,
                insert_pathing_information,
                update_movement_target,
                move_entities,
                update_current_tilepos,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        //app.add_systems(Startup, spawn_tiled_tiles);
        //app.init_asset::<TiledMap>()
        //    .register_asset_loader(TiledLoader)
        //    .add_systems(Startup, spawn_map)
        //    .add_systems(Update, process_loaded_maps);
    }
}

#[derive(Default)]
pub struct TiledCameraPlugin;

impl Plugin for TiledCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, movement.run_if(in_state(GameState::InGame)));
    }
}
