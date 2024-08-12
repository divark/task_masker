mod mock_plugins;

use crate::mock_plugins::{GameWorld, MockTiledMapPlugin};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::visual::animations::{AnimationIndices, AnimationTimer};

#[given("a Tiled Map,")]
fn spawn_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockTiledMapPlugin);
    world.update(1);

    world.app.add_plugins(PathFindingPlugin);
    world.update(1);
}

#[when("the Campfire is spawned on the Tiled Map,")]
fn spawn_campfire_on_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockEnvironmentAnimationsPlugin);

    world.update(1);
}

#[then("the Campfire should have an animation speed set,")]
fn campfire_has_animation_speed(world: &mut GameWorld) {
    world.update(1);

    let campfire_animation_speed = world.find_with::<AnimationTimer, CampfireLabel>();

    assert!(campfire_animation_speed.is_some());
}

#[then("the Campfire should have 23 frames to flicker through while animating.")]
fn campfire_has_23_frames(world: &mut GameWorld) {
    let campfire_animation_indices = world.find_with::<AnimationIndices, CampfireLabel>();

    assert_eq!(23, campfire_animation_indices.len());
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/environment.feature"));
}
