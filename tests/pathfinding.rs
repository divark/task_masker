mod mock_plugins;

use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;

use cucumber::{given, then, when, World};

use crate::mock_plugins::{GameWorld, MockTiledMapPlugin};
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::map::path_finding::{GroundGraph, UndirectedGraph};

#[given("the Tiled Loading module is loaded,")]
fn load_tiled_module(world: &mut GameWorld) {
    world.app.add_plugins(MockTiledMapPlugin);
    world.update(1);
}

#[given("the Path Finding module is loaded,")]
fn load_pathfinding_module(world: &mut GameWorld) {
    world.app.add_plugins(PathFindingPlugin);
    world.update(1);
}

#[when("the Tiled map is loaded,")]
fn wait_until_tiled_map_loaded(world: &mut GameWorld) {
    loop {
        world.update(1);
        let still_adding_tiles = world.contains_when::<TilePos, Added<TilePos>>();
        if !still_adding_tiles {
            break;
        }
    }
}

#[then("there should be an Undirected Graph representing all ground tiles.")]
fn ground_undirected_graph_should_exist(world: &mut GameWorld) {
    world.update(1);

    let found_ground_graph = world.find_with::<UndirectedGraph, GroundGraph>();

    assert!(found_ground_graph.is_some());
}

//#[then("there should be a Path from the Undirected Graph starting from one tile, going to a neighboring tile.")]
//fn simple_tile_path_exists(world: &mut GameWorld) {
//    world.update(1);
//
//    let ground_graph = world.find_with::<UndirectedGraph, GroundGraph>().expect("simple_tile_path_exists: An undirected graph representing the ground tiles does not exist.");
//    let tile_path = ground_graph.shortest_path(TilePos::new(44, 59), TilePos::new(44, 58));
//
//    assert!(tile_path.is_some());
//}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/pathfinding.feature"));
}
