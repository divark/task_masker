mod mock_plugins;

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
    world.app.update();

    world.app.add_plugins(PathFindingPlugin);
    world.app.update();
}

#[given("a Chatter spawned on the Tiled Map")]
fn spawn_chatter_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockChatterPlugin);
    world.app.update();
}

#[given("a Streamer spawned on the Tiled Map")]
fn spawn_streamer_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockStreamerPlugin);
    world.app.update();
}

#[when("the Chatter wants to speak")]
fn make_chatter_approach_to_speak(world: &mut GameWorld) {
    world.app.world.send_event(ChatMsg {
        name: String::from("Chatter"),
        msg: String::from("Hello Caveman!"),
    });

    world.app.update();
}

#[when("the Chatter has approached the Streamer")]
fn wait_for_chatter_to_approach_to_speak(world: &mut GameWorld) {
    make_chatter_approach_to_speak(world);

    loop {
        world.app.update();

        let chatter_status = world
            .app
            .world
            .query::<&ChatterStatus>()
            .get_single(&world.app.world)
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
        world.app.update();

        let chatter_status = world
            .app
            .world
            .query::<&ChatterStatus>()
            .get_single(&world.app.world)
            .expect("wait_for_chatter_to_finish_speaking: Chatter does not have a Status.");

        if *chatter_status != ChatterStatus::Speaking {
            break;
        }
    }
}

#[then("the Chatter will approach the Streamer")]
fn chatter_should_approach_to_streamer(world: &mut GameWorld) {
    world.app.update();

    let chatter_status = world
        .app
        .world
        .query::<&ChatterStatus>()
        .get_single(&world.app.world)
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Approaching);

    let chatter_path = world
        .app
        .world
        .query_filtered::<&Path, With<ChatterStatus>>()
        .get_single(&world.app.world)
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Path.");

    assert_ne!(chatter_path.len(), 0);
}

#[then("the Chatter will be two tiles away from the Streamer")]
fn chatter_should_be_two_tiles_away_from_streamer(world: &mut GameWorld) {
    world.app.update();

    let chatter_status = world
        .app
        .world
        .query::<&ChatterStatus>()
        .get_single(&world.app.world)
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Speaking);

    let chatter_tilepos = world
        .app
        .world
        .query_filtered::<&TilePos, With<ChatterLabel>>()
        .get_single(&world.app.world)
        .expect("chatter_should_be_two_tiles_away_from_streamer: Chatter does not have a TilePos.")
        .clone();

    let streamer_tilepos = world
        .app
        .world
        .query_filtered::<&TilePos, With<StreamerLabel>>()
        .get_single(&world.app.world)
        .expect("chatter_should_be_two_tiles_away_from_streamer: Streamer does not have a TilePos.")
        .clone();

    let tile_distance = distance_of(chatter_tilepos, streamer_tilepos);
    assert_eq!(tile_distance, 2);
}

#[then("the Chatter will begin to speak")]
fn chatter_should_start_speaking(world: &mut GameWorld) {
    world.app.update();

    let chatter_status = world
        .app
        .world
        .query::<&ChatterStatus>()
        .get_single(&world.app.world)
        .expect("chatter_should_start_speaking: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Speaking);
}

#[then("the Chatter leaves back to its resting point")]
fn chatter_should_be_leaving_back_to_spawn(world: &mut GameWorld) {
    loop {
        world.app.update();

        let chatter_status = world
            .app
            .world
            .query::<&ChatterStatus>()
            .get_single(&world.app.world)
            .expect("chatter_should_be_leaving_back_to_spawn: Chatter does not have a Status.");

        if *chatter_status == ChatterStatus::Idle {
            break;
        }
    }

    let (chatter_tilepos, chatter_spawn) = world
        .app
        .world
        .query_filtered::<(&TilePos, &SpawnPoint), With<ChatterStatus>>()
        .get_single(&world.app.world)
        .expect("chatter_should_be_leaving_back_to_spawn: Chatter is missing pathfinding-based information and/or Status.");

    assert_eq!(chatter_spawn.0, *chatter_tilepos);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/chatter.feature"));
}
