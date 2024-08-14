mod mock_plugins;

use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;

use task_masker::entities::fruit::*;
use task_masker::entities::streamer::*;
use task_masker::entities::TriggerQueue;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::PathFindingPlugin;

use crate::mock_plugins::{GameWorld, MockFruitPlugin, MockStreamerPlugin, MockTiledMapPlugin};

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

#[given("Fruits are spawned on the Tiled Map,")]
fn spawn_fruit_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockFruitPlugin);

    world.app.update();
}

#[when("some Fruit is requested to drop,")]
fn trigger_fruit_to_fall(world: &mut GameWorld) {
    let mut game_world = world.app.world_mut();

    let mut fruit_queue = game_world
        .query_filtered::<&mut TriggerQueue, With<FruitState>>()
        .iter_mut(&mut game_world)
        .next()
        .expect("trigger_fruit_to_fall: Could not find Fruit with Trigger Queue.");

    fruit_queue.push_back(());

    world.app.update();
}

#[when("the Fruit has been dropped,")]
fn wait_for_fruit_to_be_dropped(world: &mut GameWorld) {
    let mut game_world = world.app.world_mut();

    let mut fruit_queue = game_world
        .query_filtered::<&mut TriggerQueue, With<FruitState>>()
        .iter_mut(&mut game_world)
        .next()
        .expect("wait_for_fruit_to_be_dropped: Could not find Fruit with Trigger Queue.");

    fruit_queue.push_back(());

    loop {
        world.app.update();

        let fruit_status = world
            .app
            .world_mut()
            .query::<&FruitState>()
            .iter(&world.app.world())
            .next()
            .expect("wait_for_fruit_to_be_dropped: No piece of Fruit found with its own state.");

        if *fruit_status == FruitState::Dropped {
            break;
        }
    }
}

#[when("the Streamer is over the dropped Fruit,")]
fn wait_for_streamer_to_be_over_fruit(world: &mut GameWorld) {
    let fruit_tilepos = *world
        .app
        .world_mut()
        .query_filtered::<&TilePos, With<FruitState>>()
        .iter(&world.app.world())
        .next()
        .expect("wait_for_streamer_to_be_over_fruit: Fruit was not found with TilePos.");

    loop {
        world.app.update();

        let streamer_tilepos = *world
            .app
            .world_mut()
            .query_filtered::<&TilePos, With<StreamerLabel>>()
            .single(&world.app.world());

        if streamer_tilepos == fruit_tilepos {
            break;
        }
    }
}

#[then("the Fruit should be heading towards the ground.")]
fn fruit_should_be_falling(world: &mut GameWorld) {
    world.app.update();

    let fruit_status = world
        .app
        .world_mut()
        .query::<&FruitState>()
        .iter(&world.app.world())
        .next()
        .expect("fruit_should_be_falling: Could not find Fruit that should be falling.");

    assert_eq!(*fruit_status, FruitState::Falling);
}

#[then("the Streamer should be heading towards the fallen Fruit's position.")]
fn streamer_should_be_heading_towards_fruit(world: &mut GameWorld) {
    // We need to wait for the Streamer to actually be moving
    // in order for their Path to be populated with something.
    loop {
        world.app.update();

        let streamer_status = world
            .app
            .world_mut()
            .query::<&StreamerState>()
            .get_single(&world.app.world())
            .expect("streamer_should_be_heading_towards_fruit: Streamer does not have a State.");

        if *streamer_status == StreamerState::Moving {
            break;
        }
    }

    let streamer_path_destination = *world
        .app
        .world_mut()
        .query_filtered::<&Path, With<StreamerLabel>>()
        .get_single(&world.app.world())
        .expect("streamer_should_be_heading_towards_fruit: Streamer does not have a Path.")
        .iter()
        .last()
        .expect(
            "streamer_should_be_heading_towards_fruit: Streamer's Path does not contain anything.",
        );

    let streamer_destination_transform = world
        .app
        .world_mut()
        .query::<(&NodeData, &GraphType)>()
        .iter(&world.app.world())
        .filter(|graph_info| *graph_info.1 == GraphType::Ground)
        .map(|graph_info| graph_info.0.0[streamer_path_destination])
        .map(Transform::from_translation)
        .next()
        .expect("streamer_should_be_heading_towards_fruit: The destination Transform could not be derived from the Streamer's Path.");

    let fruit_transform = *world
        .app
        .world_mut()
        .query_filtered::<&Transform, With<FruitState>>()
        .iter(&world.app.world())
        .next()
        .expect("streamer_should_be_heading_towards_fruit: No Fruit with Transform found.");

    assert_eq!(fruit_transform, streamer_destination_transform);
}

#[then("the Fruit will re-appear back on its tree.")]
fn fruit_should_respawn(world: &mut GameWorld) {
    world.app.update();

    let fruit_state = world
        .app
        .world_mut()
        .query::<&FruitState>()
        .iter(&world.app.world())
        .next()
        .expect("fruit_should_respawn: Fruit was not found with State.");

    assert_eq!(*fruit_state, FruitState::Hanging);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/fruit.feature"));
}
