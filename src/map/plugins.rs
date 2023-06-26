use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::GameState;
use super::{
    path_finding::{
        create_ground_graph, insert_pathing_information, move_entities, move_streamer,
        move_streamer_on_spacebar, update_movement_target,
    },
    tiled::{process_loaded_maps, TiledLoader, TiledMap, despawn_map},
};

#[derive(Default)]
pub struct PathFindingPlugin;

impl Plugin for PathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TilePos>()
            .add_systems((
                create_ground_graph,
                insert_pathing_information,
                update_movement_target,
                move_entities,
                move_streamer,
                //move_streamer_on_spacebar,
        ).in_set(OnUpdate(GameState::InGame)))
        .add_system(despawn_map.in_schedule(OnExit(GameState::InGame)));
    }
}

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<TiledMap>()
            .add_asset_loader(TiledLoader)
            .add_system(process_loaded_maps.in_set(OnUpdate(GameState::InGame)));
    }
}
