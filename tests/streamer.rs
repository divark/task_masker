mod mock_plugins;

use crate::mock_plugins::{GameWorld, MockStreamerPlugin, MockTiledMapPlugin};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};
use task_masker::entities::streamer::*;
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

    world.app.world.send_event(TilePosEvent::new(tile_pos));
    loop {
        let streamer_status = *world
            .app
            .world
            .query::<&StreamerStatus>()
            .get_single(&world.app.world)
            .expect("request_streamer_to_move_to_lower_location: Streamer does not have a status.");

        if streamer_status == StreamerStatus::Moving {
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
            .world
            .query::<&StreamerStatus>()
            .get_single(&world.app.world)
            .expect(
                "streamer_should_have_reached_lower_location: Streamer does not have a Status.",
            );

        if streamer_status == StreamerStatus::Idle {
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
        .world
        .query_filtered::<&TilePos, With<StreamerLabel>>()
        .get_single(&world.app.world)
        .expect("streamer_should_have_reached_lower_location: Streamer does not have a TilePos.");

    assert_eq!(expected_tilepos, *streamer_tilepos);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/streamer.feature"));
}
