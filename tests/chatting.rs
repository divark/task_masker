mod mock_plugins;

use crate::mock_plugins::{GameWorld, MockStreamerPlugin, MockTiledMapPlugin};

use bevy::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::streamer::*;
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

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/chatting.feature"));
}
