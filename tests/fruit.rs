mod mock_plugins;

use bevy::prelude::*;

use task_masker::entities::fruit::*;
use task_masker::map::path_finding::Target;
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

#[when("a piece of Fruit is detached from its tree,")]
fn trigger_fruit_to_fall(world: &mut GameWorld) {
    let mut fruit_queue = world
        .app
        .world
        .query_filtered::<&mut TriggerQueue, With<FruitState>>()
        .iter_mut(&mut world.app.world)
        .next()
        .expect("trigger_fruit_to_fall: Could not find Fruit with Trigger Queue.");

    fruit_queue.push_back(());

    world.app.update();
}

#[then("the Fruit should be dropped on the ground.")]
fn fruit_should_be_dropped(world: &mut GameWorld) {
    loop {
        world.app.update();

        let fruit_target = world
            .app
            .world
            .query_filtered::<&Target, With<FruitState>>()
            .iter(&world.app.world)
            .next()
            .expect("fruit_should_be_dropped: No Target information was found for Fruit.");

        if fruit_target.is_none() {
            break;
        }
    }

    let fruit_state = world
        .app
        .world
        .query::<&FruitState>()
        .iter(&world.app.world)
        .next()
        .expect("fruit_should_be_dropped: No Fruit was found.");

    assert_eq!(*fruit_state, FruitState::Dropped);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/fruit.feature"));
}
