mod mock_plugins;

use bevy::prelude::*;

use task_masker::entities::crop::*;
use task_masker::entities::streamer::*;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::map::tilemap::*;

use crate::mock_plugins::{GameWorld, MockCropPlugin, MockStreamerPlugin, MockTiledMapPlugin};

use cucumber::{given, then, when, World};

#[given("a Tiled Map,")]
fn spawn_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockTiledMapPlugin);
    world.app.update();

    world.app.add_plugins(PathFindingPlugin);
    world.app.update();
}

#[given("a Streamer spawned on the Tiled Map,")]
fn spawn_streamer_on_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockStreamerPlugin);

    world.app.update();
}

#[given("Crops are spawned on the Tiled Map,")]
fn spawn_crops_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockCropPlugin);

    world.app.update();
}

#[when("the Crop has been fully grown,")]
fn grow_one_crop_fully(world: &mut GameWorld) {
    world.app.update();

    let mut game_world = world.app.world_mut();
    let mut crop_state = game_world
        .query::<&mut CropState>()
        .iter_mut(&mut game_world)
        .next()
        .expect("grow_one_crop_fully: Could not find a Crop with a State.");

    *crop_state = CropState::Grown;
}

#[when("the Streamer is over the grown Crop,")]
fn wait_for_streamer_to_be_over_crop(world: &mut GameWorld) {
    let crop_tilepos = *world
        .app
        .world_mut()
        .query_filtered::<&TileGridCoordinates, With<CropState>>()
        .iter(&world.app.world())
        .next()
        .expect("wait_for_streamer_to_be_over_crop: Crop was not found with TileGridCoordinates.");

    loop {
        world.app.update();

        let streamer_tilepos = *world
            .app
            .world_mut()
            .query_filtered::<&TileGridCoordinates, With<StreamerLabel>>()
            .single(&world.app.world());

        if streamer_tilepos == crop_tilepos {
            break;
        }
    }
}

#[then("the Streamer should be heading towards the grown Crop's position.")]
fn streamer_should_be_heading_towards_crop(world: &mut GameWorld) {
    // We need to wait for the Streamer to actually be moving
    // in order for their Path to be populated with something.
    loop {
        world.app.update();

        let streamer_status = world
            .app
            .world_mut()
            .query::<&StreamerState>()
            .get_single(&world.app.world())
            .expect("streamer_should_be_heading_towards_crop: Streamer does not have a State.");

        if *streamer_status == StreamerState::Moving {
            break;
        }
    }

    let streamer_path_destination = *world
        .app
        .world_mut()
        .query_filtered::<&Path, With<StreamerLabel>>()
        .get_single(&world.app.world())
        .expect("streamer_should_be_heading_towards_crop: Streamer does not have a Path.")
        .iter()
        .last()
        .expect(
            "streamer_should_be_heading_towards_crop: Streamer's Path does not contain anything.",
        );

    let streamer_destination_transform = world
        .app
        .world_mut()
        .query::<&UndirectedGraph>()
        .iter(&world.app.world())
        .filter(|graph| *graph.get_node_type() == GraphType::Ground)
        .map(|graph| *graph.get_node(streamer_path_destination).unwrap())
        .map(Transform::from_translation)
        .next()
        .expect("streamer_should_be_heading_towards_crop: The destination Transform could not be derived from the Streamer's Path.");

    let crop_transform = *world
        .app
        .world_mut()
        .query_filtered::<&Transform, With<CropState>>()
        .iter(&world.app.world())
        .next()
        .expect("streamer_should_be_heading_towards_crop: No Crop with Transform found.");

    assert_eq!(crop_transform, streamer_destination_transform);
}

#[then("the Crop will be replanted.")]
fn crop_should_be_planted(world: &mut GameWorld) {
    world.app.update();

    let crop_state = world
        .app
        .world_mut()
        .query::<&CropState>()
        .iter(&world.app.world())
        .next()
        .expect("crop_should_be_planted: Could not find Crop with State.");

    assert_eq!(*crop_state, CropState::Planted);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/crop.feature"));
}
