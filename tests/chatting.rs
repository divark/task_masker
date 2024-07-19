mod mock_plugins;

use crate::mock_plugins::{GameWorld, MockStreamerPlugin, MockTiledMapPlugin};

use bevy::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::ui::chatting::Msg;
use task_masker::ui::plugins::ChattingPlugin;

use task_masker::entities::streamer::*;
use task_masker::entities::MovementType;

use task_masker::map::plugins::TilePosEvent;

#[given("a Streamer is spawned on the map,")]
fn spawn_streamer(world: &mut GameWorld) {
    // A Streamer only exists in context of a map,
    // so we have to load a Tiled map as a prerequisite.
    world.app.add_plugins(MockTiledMapPlugin);
    world.app.update();

    // One way a Streamer reacts is by being told
    // where to go. The Mock plugin for the Streamer
    // has systems that depend on this fact, so we
    // include it here, despite it not being used
    // under these tests.
    world.app.add_event::<TilePosEvent>();
    world.app.add_plugins(MockStreamerPlugin);
    world.app.update();
}

#[given("the Chatting interface exists,")]
fn spawn_chatting_ui(world: &mut GameWorld) {
    world.app.add_plugins(ChattingPlugin);
    world.app.update();
}

#[when("the Streamer sends a chat message,")]
fn streamer_sends_msg(world: &mut GameWorld) {
    let streamer_msg = Msg {
        speaker_name: String::from("Caveman"),
        msg: String::from("This is a test message to see if this works and types as expected."),
        speaker_role: MovementType::Walk,
    };

    world.app.world.send_event::<Msg>(streamer_msg);
    world.app.update();
}

#[then("the Chatting Queue should contain the Streamer's chat message.")]
fn chatting_queue_has_streamer_msg(world: &mut GameWorld) {
    // TODO: Create component called MessageQueue.
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/chatting.feature"));
}
