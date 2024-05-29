use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::map::tiled::convert_tiled_to_bevy_pos;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct MapWorld {
    world_length: u32,
    world_width: u32,
    world_height: u32,

    tiled_tile_pos: TilePos,
    converted_tile_pos: TilePos,
    bevy_tile_pos: TilePos,
}

impl MapWorld {
    fn new() -> Self {
        let tiled_tile_pos = TilePos::new(0, 0);
        let bevy_tile_pos = TilePos::new(1, 1);

        Self {
            world_length: 0,
            world_width: 0,
            world_height: 0,

            tiled_tile_pos,
            converted_tile_pos: tiled_tile_pos,
            bevy_tile_pos,
        }
    }

    /// Returns a TilePos mapped to the Bevy Coordinate System
    /// from some given TilePos using the Tiled Coordinate System.
    fn map_to_bevy_tilepos(&self, tiled_tilepos: TilePos) -> TilePos {
        convert_tiled_to_bevy_pos(tiled_tilepos, self.world_width)
    }
}

#[given("a 5x5 Island Map")]
fn spawn_island_map(world: &mut MapWorld) {
    let world_size = 5;

    world.world_length = world_size;
    world.world_width = world_size;
    world.world_height = world_size;
}

#[given("a Tiled Tile Position")]
fn tiled_position(world: &mut MapWorld) {
    world.tiled_tile_pos = TilePos::new(1, 1);
}

#[given("an expected Bevy Tile Position")]
fn expected_bevy_position(world: &mut MapWorld) {
    world.bevy_tile_pos = TilePos::new(1, 3);
}

#[when("I convert the Tiled Tile Position into a Bevy Tile Position")]
fn map_tiled_to_bevy_tile_pos(world: &mut MapWorld) {
    world.converted_tile_pos = world.map_to_bevy_tilepos(world.tiled_tile_pos);
}

#[then("the converted Tile Position should match the expected Bevy Tile Position")]
fn converted_tile_matches_bevy_tile(world: &mut MapWorld) {
    assert_eq!(world.converted_tile_pos, world.bevy_tile_pos);
}

fn main() {
    futures::executor::block_on(MapWorld::run("tests/feature-files/traveling.feature"));
}
