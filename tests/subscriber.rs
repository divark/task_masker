mod mock_plugins;

use crate::mock_plugins::{
    intercept_typing_timer, reduce_wait_times_to_zero, GameWorld, MockChattingPlugin,
    MockStreamerPlugin, MockSubscriberPlugin, MockTiledMapPlugin,
};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::subscriber::*;
use task_masker::entities::WaitToLeaveTimer;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::ui::chatting::TypingMsg;

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

#[given("the Chatting interface exists")]
fn spawn_chatting_ui(world: &mut GameWorld) {
    world.app.add_plugins(MockChattingPlugin);
    world.app.add_systems(Update, intercept_typing_timer);
    world.update(1);
}

#[when("the Subscriber wants to speak")]
fn make_subscriber_approach_to_speak(world: &mut GameWorld) {
    world.app.world_mut().send_event(SubscriberMsg {
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
            .world_mut()
            .query::<&SubscriberStatus>()
            .get_single(&world.app.world())
            .expect("wait_for_subscriber_to_approach_to_speak: Subscriber does not have a Status.");

        if *subscriber_status == SubscriberStatus::Speaking {
            break;
        }
    }
}

#[when("the Subscriber sends a long chat message")]
fn subscriber_sends_long_msg(world: &mut GameWorld) {
    let long_msg = String::from("So, if you're learning a subject of math for the first time, it's helpful to actually learn about the concepts behind it before going into the course, since you're otherwise being overloaded with a bunch of terminology. Doing it this way, it's important to do so with the angle of finding how it's important to your work, using analogies and metaphors to make the knowledge personal");

    let subscriber_msg = SubscriberMsg {
        name: String::from("Subscriber"),
        msg: long_msg,
    };

    world.broadcast_event::<SubscriberMsg>(subscriber_msg);
    world.update(1);
}

#[when("the Subscriber is done speaking")]
fn wait_for_subscriber_to_finish_speaking(world: &mut GameWorld) {
    world.app.add_systems(Update, reduce_wait_times_to_zero);

    loop {
        world.app.update();

        let subscriber_status = world
            .app
            .world_mut()
            .query::<&SubscriberStatus>()
            .get_single(&world.app.world())
            .expect("wait_for_subscriber_to_finish_speaking: Subscriber does not have a Status.");

        if *subscriber_status != SubscriberStatus::Speaking {
            break;
        }
    }
}

#[when("the Subscriber is almost done speaking to the Streamer")]
fn wait_until_subscriber_near_end_of_speaking(world: &mut GameWorld) {
    loop {
        world.update(1);

        let typing_msg = world.find::<TypingMsg>();
        if typing_msg.is_none() {
            continue;
        }

        let msg_index = typing_msg
            .expect(
                "wait_until_subscriber_near_end_of_speaking: Could not find Typing Indicator type.",
            )
            .idx();
        if msg_index > 356 {
            break;
        }
    }
}

#[then("the Subscriber will be on the coast closest to the Streamer")]
fn subscriber_should_be_near_coast_closest_to_streamer(world: &mut GameWorld) {
    world.app.update();

    let subscriber_tilepos = world
        .app
        .world_mut()
        .query_filtered::<&TilePos, With<SubscriberLabel>>()
        .get_single(&world.app.world())
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

    let ground_nodes = world
        .app
        .world_mut()
        .query::<&UndirectedGraph>()
        .iter(&world.app.world())
        .find(|graph| *graph.get_node_type() == GraphType::Ground)
        .expect("subscriber_should_be_near_coast_closest_to_streamer: Ground Graph does not have Node Edges. Does the Graph not exist?");

    let subscriber_next_to_coast = subscriber_tilepos_neighbor_indexes
        .iter()
        .any(|index| !ground_nodes.edges()[*index].is_empty());

    assert!(subscriber_next_to_coast);
}

#[then("the Subscriber will approach the Streamer")]
fn subscriber_should_approach_to_streamer(world: &mut GameWorld) {
    world.app.update();

    let subscriber_status = world
        .app
        .world_mut()
        .query::<&SubscriberStatus>()
        .get_single(&world.app.world())
        .expect("subscriber_should_approach_to_streamer: Subscriber does not have a Status.");

    assert_eq!(*subscriber_status, SubscriberStatus::Approaching);

    let subscriber_path = world
        .app
        .world_mut()
        .query_filtered::<&Path, With<SubscriberStatus>>()
        .get_single(&world.app.world())
        .expect("subscriber_should_approach_to_streamer: Subscriber does not have a Path.");

    assert_ne!(subscriber_path.len(), 0);
}

#[then("the Subscriber will begin to speak")]
fn subscriber_should_start_speaking(world: &mut GameWorld) {
    world.app.update();

    let subscriber_status = world
        .app
        .world_mut()
        .query::<&SubscriberStatus>()
        .get_single(&world.app.world())
        .expect("subscriber_should_start_speaking: Subscriber does not have a Status.");

    assert_eq!(*subscriber_status, SubscriberStatus::Speaking);
}

#[then("the Subscriber should still be speaking")]
fn subscriber_should_still_be_speaking(world: &mut GameWorld) {
    world.update(1);

    let msg_is_still_being_typed = !world
        .find::<TypingMsg>()
        .expect("subscriber_should_still_be_speaking: Typing Indicator could not be found")
        .at_end();
    assert!(msg_is_still_being_typed);

    let expected_subscriber_status = SubscriberStatus::Speaking;
    let actual_subscriber_status = world
        .find::<SubscriberStatus>()
        .expect("subscriber_should_still_be_speaking: Subscriber status could not be found.");

    assert_eq!(expected_subscriber_status, *actual_subscriber_status);

    let has_waiting_timer = world.find::<WaitToLeaveTimer>().is_some();
    assert!(!has_waiting_timer);
}

#[then("the Subscriber leaves back to its resting point")]
fn subscriber_should_be_leaving_back_to_spawn(world: &mut GameWorld) {
    loop {
        world.app.update();

        let subscriber_status = world
            .app
            .world_mut()
            .query::<&SubscriberStatus>()
            .get_single(&world.app.world())
            .expect(
                "subscriber_should_be_leaving_back_to_spawn: Subscriber does not have a Status.",
            );

        if *subscriber_status == SubscriberStatus::Idle {
            break;
        }
    }

    let (subscriber_tilepos, subscriber_spawn) = world
        .app
        .world_mut()
        .query_filtered::<(&TilePos, &SpawnPoint), With<SubscriberStatus>>()
        .get_single(&world.app.world())
        .expect("subscriber_should_be_leaving_back_to_spawn: Subscriber is missing pathfinding-based information and/or Status.");

    assert_eq!(subscriber_spawn.0, *subscriber_tilepos);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/subscriber.feature"));
}
