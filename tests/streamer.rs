mod mock_plugins;

use crate::mock_plugins::{
    GameWorld, MockEnvironmentAnimationsPlugin, MockStreamerPlugin, MockTiledMapPlugin,
};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};
use task_masker::entities::streamer::*;
use task_masker::map::path_finding::Direction;
use task_masker::map::plugins::{PathFindingPlugin, TilePosEvent};

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

#[given("the Campfire spawned on the Tiled Map,")]
fn spawn_campfire_on_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockEnvironmentAnimationsPlugin);

    world.update(1);
}

#[when(regex = r"^the Status changes to (Online|Away),")]
fn request_status_change(world: &mut GameWorld, status_option: String) {
    let new_status = match status_option.as_str() {
        "Online" => OnlineStatus::Online,
        "Away" => OnlineStatus::Away,
        _ => unreachable!(),
    };

    world.broadcast_event(new_status);
    world.update(1);
}

#[when("the Streamer is done traveling,")]
fn check_streamer_is_done_traveling(world: &mut GameWorld) {
    loop {
        world.update(1);

        let streamer_status = world.find::<StreamerState>().expect(
            "check_streamer_is_done_traveling: Could not find Streamer with StreamerStatus.",
        );

        if *streamer_status == StreamerState::Idle {
            break;
        }
    }
}

#[when(
    regex = r"^the Streamer is requested to travel to an? (lower|equal in height|higher) location,"
)]
fn request_streamer_to_move_to_lower_location(world: &mut GameWorld, option: String) {
    let tile_pos = match option.as_str() {
        "lower" => TilePos::new(64, 47),
        "equal in height" => TilePos::new(45, 40),
        "higher" => TilePos::new(44, 35),
        _ => unreachable!(),
    };

    world
        .app
        .world_mut()
        .send_event(TilePosEvent::new(tile_pos));
    loop {
        let streamer_status = *world
            .app
            .world_mut()
            .query::<&StreamerState>()
            .get_single(&world.app.world())
            .expect("request_streamer_to_move_to_lower_location: Streamer does not have a status.");

        if streamer_status == StreamerState::Moving {
            break;
        }

        world.app.update();
    }
}

#[then(
    regex = r"^the Streamer will arrive at the (lower|equal in height|higher) location after traveling there."
)]
fn streamer_should_have_reached_lower_location(world: &mut GameWorld, option: String) {
    loop {
        world.app.update();

        let streamer_status = *world
            .app
            .world_mut()
            .query::<&StreamerState>()
            .get_single(&world.app.world())
            .expect(
                "streamer_should_have_reached_lower_location: Streamer does not have a Status.",
            );

        if streamer_status == StreamerState::Idle {
            break;
        }
    }

    let expected_tilepos = match option.as_str() {
        "lower" => TilePos::new(64, 47),
        "equal in height" => TilePos::new(45, 40),
        "higher" => TilePos::new(44, 35),
        _ => unreachable!(),
    };

    let streamer_tilepos = world
        .app
        .world_mut()
        .query_filtered::<&TilePos, With<StreamerLabel>>()
        .get_single(&world.app.world())
        .expect("streamer_should_have_reached_lower_location: Streamer does not have a TilePos.");

    assert_eq!(expected_tilepos, *streamer_tilepos);
}

#[then(regex = r"^the Streamer should be (to the left of the campfire|in the cave).")]
fn streamer_should_have_reached_specific_location(world: &mut GameWorld, location_option: String) {
    let expected_location = match location_option.as_str() {
        "to the left of the campfire" => TilePos::new(41, 100 - 50 - 1),
        "in the cave" => TilePos::new(39, 100 - 59 - 1),
        _ => unreachable!(),
    };

    let actual_location = world.find_with::<TilePos, StreamerLabel>().expect(
        "streamer_should_have_reached_specific_location: Streamer does not have a TilePos.",
    );

    assert_eq!(expected_location, *actual_location);
}

#[then("the Streamer should be facing right towards the campfire.")]
fn streamer_should_be_facing_towards_campfire(world: &mut GameWorld) {
    world.update(1);

    let expected_direction = Direction::BottomRight;
    let actual_direction = world
        .find_with::<Direction, StreamerLabel>()
        .expect("streamer_should_be_facing_towards_campfire: Streamer does not have a Direction.");

    assert_eq!(expected_direction, *actual_direction);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/streamer.feature"));
}
