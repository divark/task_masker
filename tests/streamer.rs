use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};
use task_masker::entities::streamer::*;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::{PathFindingPlugin, TilePosEvent};
use task_masker::map::tiled::*;
use task_masker::GameState;

#[derive(Default)]
pub struct MockTiledMapPlugin;

impl Plugin for MockTiledMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tiles_from_tiledmap);
    }
}

#[derive(Default)]
pub struct MockStreamerPlugin;

impl Plugin for MockStreamerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_player_tile,
                move_streamer,
                queue_destination_for_streamer.after(spawn_player_tile),
                make_streamer_idle_when_not_moving,
                update_status_when_speaking,
            ),
        );
    }
}

fn intercept_movement_timer(mut timer_query: Query<&mut MovementTimer, Added<MovementTimer>>) {
    for mut movement_timer in &mut timer_query {
        movement_timer.0 = Timer::new(Duration::from_secs(0), TimerMode::Repeating);
    }
}

#[derive(Debug, World)]
#[world(init = Self::new)]
struct GameWorld {
    app: App,
}

impl GameWorld {
    fn new() -> Self {
        let mut app = App::new();

        app.init_state::<GameState>();
        app.insert_state(GameState::InGame);
        app.add_plugins(MinimalPlugins);

        app.add_systems(Update, intercept_movement_timer);

        Self { app }
    }
}

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
        let streamer_status = world
            .app
            .world
            .query::<&StreamerStatus>()
            .get_single(&world.app.world)
            .expect("request_streamer_to_move_to_lower_location: Streamer does not have a status.")
            .clone();

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

        let streamer_status = world
            .app
            .world
            .query::<&StreamerStatus>()
            .get_single(&world.app.world)
            .expect("streamer_should_have_reached_lower_location: Streamer does not have a Status.")
            .clone();

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
