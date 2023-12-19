use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use super::{
    path_finding::*,
    tiled::{process_loaded_maps, TiledLoader, TiledMap},
};
use crate::{spawn_map, GameState};

#[derive(Event)]
pub struct TilePosEvent {
    pub destination: TilePos,
}

impl TilePosEvent {
    pub fn new(destination: TilePos) -> Self {
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
                insert_pathing_information,
                update_movement_target,
                move_entities,
                move_streamer,
                move_streamer_on_spacebar,
                queue_destination_for_streamer,
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
        app.init_asset::<TiledMap>()
            .register_asset_loader(TiledLoader)
            .add_systems(Startup, spawn_map)
            .add_systems(Update, process_loaded_maps);
    }
}
