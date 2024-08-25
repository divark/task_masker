mod mock_plugins;

use futures::executor::block_on;

use crate::mock_plugins::{
    reduce_wait_times_to_zero, GameWorld, MockChatterPlugin, MockStreamerPlugin, MockTiledMapPlugin,
};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::chatter::*;
use task_masker::entities::streamer::*;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::PathFindingPlugin;

/// Returns the approximate number of Tiles away the target_pos
/// is from source_pos
fn distance_of(source_pos: TilePos, target_pos: TilePos) -> usize {
    let x1 = source_pos.x as f32;
    let x2 = target_pos.x as f32;

    let y1 = source_pos.y as f32;
    let y2 = target_pos.y as f32;

    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt().floor() as usize
}

#[given("a Tiled Map")]
fn spawn_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockTiledMapPlugin);
    world.update(1);

    world.app.add_plugins(PathFindingPlugin);
    world.update(1);
}

#[given("a Chatter spawned on the Tiled Map")]
fn spawn_chatter_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockChatterPlugin);
    world.update(1);
}

#[given("a Streamer spawned on the Tiled Map")]
fn spawn_streamer_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockStreamerPlugin);
    world.update(1);
}

#[when("the Chatter wants to speak")]
fn make_chatter_approach_to_speak(world: &mut GameWorld) {
    world.broadcast_event(ChatMsg {
        name: String::from("Chatter"),
        msg: String::from("Hello Caveman!"),
    });

    world.update(1);
}

#[when("the Chatter has approached the Streamer")]
fn wait_for_chatter_to_approach_to_speak(world: &mut GameWorld) {
    make_chatter_approach_to_speak(world);

    loop {
        world.update(1);

        let chatter_status = world
            .find::<ChatterStatus>()
            .expect("wait_for_chatter_to_approach_to_speak: Chatter does not have a Status.");

        if *chatter_status == ChatterStatus::Speaking {
            break;
        }
    }
}

#[when("the Chatter is done speaking")]
fn wait_for_chatter_to_finish_speaking(world: &mut GameWorld) {
    world.app.add_systems(Update, reduce_wait_times_to_zero);

    loop {
        world.update(1);

        let chatter_status = world
            .find::<ChatterStatus>()
            .expect("wait_for_chatter_to_finish_speaking: Chatter does not have a Status.");

        if *chatter_status != ChatterStatus::Speaking {
            break;
        }
    }
}

#[then("the Chatter will approach the Streamer")]
fn chatter_should_approach_to_streamer(world: &mut GameWorld) {
    world.update(1);

    let chatter_status = world
        .find::<ChatterStatus>()
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Approaching);

    let chatter_path = world
        .find_with::<Path, ChatterStatus>()
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Path.");

    assert_ne!(chatter_path.len(), 0);
}

#[then("the Chatter will be two tiles away from the Streamer")]
fn chatter_should_be_two_tiles_away_from_streamer(world: &mut GameWorld) {
    world.update(1);

    let chatter_status = world
        .find::<ChatterStatus>()
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Speaking);

    let chatter_tilepos = *world
        .find_with::<TilePos, ChatterLabel>()
        .expect("chatter_should_be_two_tiles_away_from_streamer: Chatter does not have a TilePos.");

    let streamer_tilepos = *world.find_with::<TilePos, StreamerLabel>().expect(
        "chatter_should_be_two_tiles_away_from_streamer: Streamer does not have a TilePos.",
    );

    let tile_distance = distance_of(chatter_tilepos, streamer_tilepos);
    assert_eq!(tile_distance, 2);
}

#[then("the Chatter will begin to speak")]
fn chatter_should_start_speaking(world: &mut GameWorld) {
    world.update(1);

    let chatter_status = world
        .find::<ChatterStatus>()
        .expect("chatter_should_start_speaking: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Speaking);
}

#[then("the Chatter leaves back to its resting point")]
fn chatter_should_be_leaving_back_to_spawn(world: &mut GameWorld) {
    loop {
        world.update(1);

        let chatter_status = world
            .find::<ChatterStatus>()
            .expect("chatter_should_be_leaving_back_to_spawn: Chatter does not have a Status.");

        if *chatter_status == ChatterStatus::Idle {
            break;
        }
    }

    let (chatter_tilepos, chatter_spawn) = world
        .app
        .world_mut()
        .query_filtered::<(&TilePos, &SpawnPoint), With<ChatterStatus>>()
        .get_single(&world.app.world())
        .expect("chatter_should_be_leaving_back_to_spawn: Chatter is missing pathfinding-based information and/or Status.");

    assert_eq!(chatter_spawn.0, *chatter_tilepos);
}

fn main() {
    block_on(GameWorld::run("tests/feature-files/chatter.feature"));
}
