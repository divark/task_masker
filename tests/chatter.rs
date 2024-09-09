mod mock_plugins;

use futures::executor::block_on;
use task_masker::entities::WaitToLeaveTimer;
use task_masker::ui::chatting::TypingMsg;

use crate::mock_plugins::{
    intercept_typing_timer, reduce_wait_times_to_zero, GameWorld, MockChatterPlugin,
    MockChattingPlugin, MockStreamerPlugin, MockTiledMapPlugin,
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

#[given("the Chatting interface exists")]
fn spawn_chatting_ui(world: &mut GameWorld) {
    world.app.add_plugins(MockChattingPlugin);
    world.app.add_systems(Update, intercept_typing_timer);
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

#[when(regex = r"the Chatter sends a(( different )|( long )| )?chat message")]
fn chatter_sends_long_msg(world: &mut GameWorld, msg_type: String) {
    let msg_contents = if msg_type.contains("long") {
        String::from("So, if you're learning a subject of math for the first time, it's helpful to actually learn about the concepts behind it before going into the course, since you're otherwise being overloaded with a bunch of terminology. Doing it this way, it's important to do so with the angle of finding how it's important to your work, using analogies and metaphors to make the knowledge personal")
    } else if msg_type.contains("different") {
        String::from("How are you doing?")
    } else {
        String::from("Hello caveman!")
    };

    let chatter_msg = ChatMsg {
        name: String::from("Birdo"),
        msg: msg_contents,
    };

    world.broadcast_event::<ChatMsg>(chatter_msg);
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

#[when("the Chatter is almost done speaking to the Streamer")]
fn wait_until_chatter_near_end_of_speaking(world: &mut GameWorld) {
    loop {
        world.update(1);

        let typing_msg = world.find::<TypingMsg>();
        if typing_msg.is_none() {
            continue;
        }

        let msg_index = typing_msg
            .expect(
                "wait_until_chatter_near_end_of_speaking: Could not find Typing Indicator type.",
            )
            .idx();
        if msg_index > 356 {
            break;
        }
    }
}

#[when("the Chatter has finished speaking the first message to the Streamer")]
fn wait_until_chatter_done_speaking_first_msg(world: &mut GameWorld) {
    loop {
        world.update(1);

        let currently_typed_msg = world.find::<TypingMsg>().expect("wait_until_chatter_done_speaking_first_msg: Chatter should be speaking, but message being typed could not be found.");
        if currently_typed_msg.at_end() {
            break;
        }
    }
}

#[then("the Chatter should not be waiting to leave")]
fn chatter_is_not_leaving(world: &mut GameWorld) {
    world.update(1);

    let chatter_waiting_to_leave = world.find::<WaitToLeaveTimer>();
    assert!(chatter_waiting_to_leave.is_none());
}

#[then("the Chatter should start speaking from the next chat message")]
fn chatter_starts_speaking_next_msg(world: &mut GameWorld) {
    world.update(1);

    let currently_typing_msg = world.find::<TypingMsg>().expect("chatter_starts_speaking_next_msg: Chatter should be speaking, but could not find a message being typed currently.");
    assert_eq!(
        currently_typing_msg.contents(),
        String::from("How are you doing?")
    );
}

#[then("the Chatter should still be speaking")]
fn chatter_should_still_be_speaking(world: &mut GameWorld) {
    world.update(1);

    let msg_is_still_being_typed = !world
        .find::<TypingMsg>()
        .expect("chatter_should_still_be_speaking: Typing Indicator could not be found")
        .at_end();
    assert!(msg_is_still_being_typed);

    let expected_chatter_status = ChatterStatus::Speaking;
    let actual_chatter_status = world
        .find::<ChatterStatus>()
        .expect("chatter_should_still_be_speaking: Chatter status could not be found.");

    assert_eq!(expected_chatter_status, *actual_chatter_status);

    let has_waiting_timer = world.find::<WaitToLeaveTimer>().is_some();
    assert!(!has_waiting_timer);
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
