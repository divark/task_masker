use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_ecs_tilemap::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::chatter::*;
use task_masker::entities::streamer::*;
use task_masker::map::path_finding::*;
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::map::tiled::spawn_tiles_from_tiledmap;
use task_masker::ui::chatting::Msg;
use task_masker::GameState;

#[derive(Default)]
pub struct MockChatterPlugin;

impl Plugin for MockChatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMsg>();
        app.add_event::<Msg>();
        app.add_systems(
            Update,
            (
                replace_chatter_tile,
                fly_to_streamer_to_speak,
                speak_to_streamer_from_chatter,
                leave_from_streamer_from_chatter.after(speak_to_streamer_from_chatter),
                return_chatter_to_idle.after(leave_from_streamer_from_chatter),
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

/// Intercepts and sets the Wait Timer interval to 0 seconds for testing purposes.
fn reduce_wait_times_to_zero(mut waiting_timers: Query<&mut WaitTimer, Added<WaitTimer>>) {
    for mut waiting_timer in &mut waiting_timers {
        waiting_timer.0 = Timer::new(Duration::from_secs(0), TimerMode::Once);
    }
}

/// Returns the approximate number of Tiles away the target_pos
/// is from source_pos
fn distance_of(source_pos: TilePos, target_pos: TilePos) -> usize {
    let x1 = source_pos.x as f32;
    let x2 = target_pos.x as f32;

    let y1 = source_pos.y as f32;
    let y2 = target_pos.y as f32;

    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt().floor() as usize
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

        let (chatter_path, chatter_target) = world
            .app
            .world
            .query_filtered::<(&Path, &Target), With<ChatterStatus>>()
            .get_single(&world.app.world)
            .expect("wait_for_chatter_to_approach_to_speak: Chatter does not have a path.");

        if chatter_path.len() == 0 && chatter_target.is_none() {
            break;
        }
    }
}

#[when("the Chatter is done speaking")]
fn wait_for_chatter_to_finish_speaking(world: &mut GameWorld) {
    world.app.add_systems(Update, reduce_wait_times_to_zero);

    loop {
        world.app.update();

        let chatter_status = world
            .app
            .world
            .query::<&ChatterStatus>()
            .get_single(&world.app.world)
            .expect("wait_for_chatter_to_finish_speaking: Chatter does not have a Status.");

        if *chatter_status != ChatterStatus::Speaking {
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
        .get_single(&world.app.world)
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Approaching);

    let chatter_path = world
        .app
        .world
        .query_filtered::<&Path, With<ChatterStatus>>()
        .get_single(&world.app.world)
        .expect("chatter_should_approach_to_streamer: Chatter does not have a Path.");

    assert_ne!(chatter_path.len(), 0);
}

#[then("the Chatter will be two tiles away from the Streamer")]
fn chatter_should_be_two_tiles_away_from_streamer(world: &mut GameWorld) {
    world.app.update();

    let chatter_tilepos = world
        .app
        .world
        .query_filtered::<&TilePos, With<ChatterLabel>>()
        .get_single(&world.app.world)
        .expect("chatter_should_be_two_tiles_away_from_streamer: Chatter does not have a TilePos.")
        .clone();

    let streamer_tilepos = world
        .app
        .world
        .query_filtered::<&TilePos, With<StreamerLabel>>()
        .get_single(&world.app.world)
        .expect("chatter_should_be_two_tiles_away_from_streamer: Streamer does not have a TilePos.")
        .clone();

    let tile_distance = distance_of(chatter_tilepos, streamer_tilepos);
    assert_eq!(tile_distance, 2);
}

#[then("the Chatter will begin to speak")]
fn chatter_should_start_speaking(world: &mut GameWorld) {
    world.app.update();

    let chatter_status = world
        .app
        .world
        .query::<&ChatterStatus>()
        .get_single(&world.app.world)
        .expect("chatter_should_start_speaking: Chatter does not have a Status.");

    assert_eq!(*chatter_status, ChatterStatus::Speaking);
}

#[then("the Chatter leaves back to its resting point")]
fn chatter_should_be_leaving_back_to_spawn(world: &mut GameWorld) {
    loop {
        world.app.update();

        let chatter_status = world
            .app
            .world
            .query::<&ChatterStatus>()
            .get_single(&world.app.world)
            .expect("chatter_should_be_leaving_back_to_spawn: Chatter does not have a Status.");

        if *chatter_status == ChatterStatus::Idle {
            break;
        }
    }

    let (chatter_tilepos, chatter_spawn) = world
        .app
        .world
        .query_filtered::<(&TilePos, &SpawnPoint), With<ChatterStatus>>()
        .get_single(&world.app.world)
        .expect("chatter_should_be_leaving_back_to_spawn: Chatter is missing pathfinding-based information and/or Status.");

    assert_eq!(chatter_spawn.0, *chatter_tilepos);
}

fn main() {
    futures::executor::block_on(GameWorld::run("tests/feature-files/chatter.feature"));
}
