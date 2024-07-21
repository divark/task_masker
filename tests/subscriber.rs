mod mock_plugins;

use crate::mock_plugins::{
    reduce_wait_times_to_zero, GameWorld, MockStreamerPlugin, MockSubscriberPlugin,
    MockTiledMapPlugin,
};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::subscriber::*;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::PathFindingPlugin;

#[given("a Tiled Map")]
fn spawn_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockTiledMapPlugin);
    world.app.update();

    world.app.add_plugins(PathFindingPlugin);
    world.app.update();
}

#[given("a Subscriber spawned on the Tiled Map")]
fn spawn_subscriber_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockSubscriberPlugin);
    world.app.update();
}

#[given("a Streamer spawned on the Tiled Map")]
fn spawn_streamer_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockStreamerPlugin);
    world.app.update();
}

#[when("the Subscriber wants to speak")]
fn make_subscriber_approach_to_speak(world: &mut GameWorld) {
    world.app.world.send_event(SubscriberMsg {
        name: String::from("Subscriber"),
        msg: String::from("Hello Caveman!"),
    });

    world.app.update();
}

#[when("the Subscriber has approached the Streamer")]
fn wait_for_subscriber_to_approach_to_speak(world: &mut GameWorld) {
    make_subscriber_approach_to_speak(world);

    loop {
        world.app.update();

        let subscriber_status = world
            .app
            .world
            .query::<&SubscriberStatus>()
            .get_single(&world.app.world)
            .expect("wait_for_subscriber_to_approach_to_speak: Subscriber does not have a Status.");

        if *subscriber_status == SubscriberStatus::Speaking {
            break;
        }
    }
}

#[when("the Subscriber is done speaking")]
fn wait_for_subscriber_to_finish_speaking(world: &mut GameWorld) {
    world.app.add_systems(Update, reduce_wait_times_to_zero);

    loop {
        world.app.update();

        let subscriber_status = world
            .app
            .world
            .query::<&SubscriberStatus>()
            .get_single(&world.app.world)
            .expect("wait_for_subscriber_to_finish_speaking: Subscriber does not have a Status.");

        if *subscriber_status != SubscriberStatus::Speaking {
            break;
        }
    }
}

#[then("the Subscriber will be on the coast closest to the Streamer")]
fn subscriber_should_be_near_coast_closest_to_streamer(world: &mut GameWorld) {
    world.app.update();

    let subscriber_tilepos = world
        .app
        .world
        .query_filtered::<&TilePos, With<SubscriberLabel>>()
        .get_single(&world.app.world)
        .expect("subscriber_should_be_near_coast_closest_to_streamer: Subscriber does not have a TilePos.");

    let subscriber_tilepos_neighbors = [
        TilePos::new(subscriber_tilepos.x, subscriber_tilepos.y - 1),
        TilePos::new(subscriber_tilepos.x, subscriber_tilepos.y + 1),
        TilePos::new(subscriber_tilepos.x - 1, subscriber_tilepos.y),
        TilePos::new(subscriber_tilepos.x + 1, subscriber_tilepos.y),
    ];

    let subscriber_tilepos_neighbor_indexes = subscriber_tilepos_neighbors
        .iter()
        .map(|tilepos| tilepos_to_idx(tilepos.x, tilepos.y, 100))
        .collect::<Vec<usize>>();

    let ground_node_neighbors = world
        .app
        .world
        .query::<(&NodeEdges, &GraphType)>()
        .iter(&world.app.world)
        .find(|graph_data| *graph_data.1 == GraphType::Ground)
        .map(|graph_data| graph_data.0)
        .expect("subscriber_should_be_near_coast_closest_to_streamer: Ground Graph does not have Node Edges. Does the Graph not exist?");

    let subscriber_next_to_coast = subscriber_tilepos_neighbor_indexes
        .iter()
        .any(|index| !ground_node_neighbors.0[*index].is_empty());

    assert!(subscriber_next_to_coast);
}

#[then("the Subscriber will approach the Streamer")]
fn subscriber_should_approach_to_streamer(world: &mut GameWorld) {
    world.app.update();

    let subscriber_status = world
        .app
        .world
        .query::<&SubscriberStatus>()
        .get_single(&world.app.world)
        .expect("subscriber_should_approach_to_streamer: Subscriber does not have a Status.");

    assert_eq!(*subscriber_status, SubscriberStatus::Approaching);

    let subscriber_path = world
        .app
        .world
        .query_filtered::<&Path, With<SubscriberStatus>>()
        .get_single(&world.app.world)
        .expect("subscriber_should_approach_to_streamer: Subscriber does not have a Path.");

    assert_ne!(subscriber_path.len(), 0);
}

#[then("the Subscriber will begin to speak")]
fn subscriber_should_start_speaking(world: &mut GameWorld) {
    world.app.update();

    let subscriber_status = world
        .app
        .world
        .query::<&SubscriberStatus>()
        .get_single(&world.app.world)
        .expect("subscriber_should_start_speaking: Subscriber does not have a Status.");

    assert_eq!(*subscriber_status, SubscriberStatus::Speaking);
}

#[then("the Subscriber leaves back to its resting point")]
fn subscriber_should_be_leaving_back_to_spawn(world: &mut GameWorld) {
    loop {
        world.app.update();

        let subscriber_status = world
            .app
            .world
            .query::<&SubscriberStatus>()
            .get_single(&world.app.world)
            .expect(
                "subscriber_should_be_leaving_back_to_spawn: Subscriber does not have a Status.",
            );

        if *subscriber_status == SubscriberStatus::Idle {
            break;
        }
    }

    let (subscriber_tilepos, subscriber_spawn) = world
        .app
        .world
        .query_filtered::<(&TilePos, &SpawnPoint), With<SubscriberStatus>>()
        .get_single(&world.app.world)
        .expect("subscriber_should_be_leaving_back_to_spawn: Subscriber is missing pathfinding-based information and/or Status.");

    assert_eq!(subscriber_spawn.0, *subscriber_tilepos);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/subscriber.feature"));
}
