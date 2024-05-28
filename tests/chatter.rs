use bevy::prelude::*;
use bevy::utils::Duration;
use cucumber::{given, then, when, World};

use task_masker::entities::chatter::*;
use task_masker::entities::streamer::*;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::map::tiled::spawn_tiles_from_tiledmap;
use task_masker::GameState;

#[derive(Default)]
pub struct MockChatterPlugin;

impl Plugin for MockChatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMsg>();
        app.add_systems(
            Update,
            (
                replace_chatter_tile,
                fly_to_streamer_to_speak,
                leave_from_streamer_from_chatter,
                return_chatter_to_idle,
                follow_streamer_while_speaking,
                follow_streamer_while_approaching_for_chatter,
            ),
        );
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
                queue_destination_for_streamer,
                update_status_when_speaking,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockTiledMapPlugin;

impl Plugin for MockTiledMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tiles_from_tiledmap);
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

#[given("a Tiled Map")]
fn spawn_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockTiledMapPlugin);
    world.app.update();

    world.app.add_plugins(PathFindingPlugin);
    world.app.update();
}

#[given("a Chatter spawned on the Tiled Map")]
fn spawn_chatter_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockChatterPlugin);
    world.app.update();
}

#[given("a Streamer spawned on the Tiled Map")]
fn spawn_streamer_from_tiled_map(world: &mut GameWorld) {
    world.app.add_plugins(MockStreamerPlugin);
    world.app.update();
}

#[when("the Chatter wants to speak")]
fn make_chatter_approach_to_speak(world: &mut GameWorld) {
    world.app.world.send_event(ChatMsg {
        name: String::from("Chatter"),
        msg: String::from("Hello Caveman!"),
    });

    world.app.update();
}

#[when("the Chatter has approached the Streamer")]
fn wait_for_chatter_to_approach_to_speak(world: &mut GameWorld) {
    make_chatter_approach_to_speak(world);

    loop {
        world.app.update();

        let chatter_path = world
            .app
            .world
            .query_filtered::<&Path, With<ChatterStatus>>()
            .iter(&world.app.world)
            .next()
            .expect("wait_for_chatter_to_approach_to_speak: Chatter does not have a path.");

        if chatter_path.len() == 0 {
            break;
        }
    }
}

#[then("the Chatter will approach the Streamer")]
fn chatter_should_approach_to_streamer(world: &mut GameWorld) {
    world.app.update();

    let chatter_status = world
        .app
        .world
        .query::<&ChatterStatus>()
        .iter(&world.app.world)
        .next()
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Approaching);

    let chatter_path = world
        .app
        .world
        .query_filtered::<&Path, With<ChatterStatus>>()
        .iter(&world.app.world)
        .next()
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Path.");

    assert_ne!(chatter_path.len(), 0);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/chatter.feature"));
}
