use bevy::prelude::*;

use bevy::ecs::query::QueryFilter;

use bevy::state::app::StatesPlugin;
use bevy::utils::Duration;

use task_masker::entities::chatter::*;
use task_masker::entities::crop::*;
use task_masker::entities::fruit::*;
use task_masker::entities::streamer::*;
use task_masker::entities::subscriber::*;
use task_masker::entities::WaitToLeaveTimer;
use task_masker::map::path_finding::*;
use task_masker::map::tiled::*;
use task_masker::ui::chatting::*;
use task_masker::ui::portrait_preferences::*;
use task_masker::ui::screens::spawn_ingame_screen;
use task_masker::visual::animations::*;
use task_masker::visual::environment::*;

use task_masker::GameState;

use cucumber::World;

#[derive(Default)]
pub struct MockChattingPlugin;

impl Plugin for MockChattingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_ingame_screen.run_if(run_once()));
        app.add_event::<Msg>().add_systems(
            Update,
            (
                insert_chatting_information,
                load_msg_into_queue.after(insert_chatting_information),
                load_queued_msg_into_textfield.after(load_msg_into_queue),
                teletype_current_message.after(load_queued_msg_into_textfield),
                activate_waiting_timer.after(teletype_current_message),
                unload_msg_on_timeup.after(activate_waiting_timer),
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

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
                add_chat_msg_to_queue.after(replace_chatter_tile),
                fly_to_streamer_to_speak.after(add_chat_msg_to_queue),
                speak_to_streamer_from_chatter.after(fly_to_streamer_to_speak),
                chatter_waits_to_leave_from_streamer.after(speak_to_streamer_from_chatter),
                leave_from_streamer_from_chatter.after(chatter_waits_to_leave_from_streamer),
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
        app.add_event::<OnlineStatus>();
        app.add_systems(
            Update,
            (
                spawn_player_tile,
                move_streamer,
                move_streamer_on_status_change,
                queue_destination_for_streamer.after(spawn_player_tile),
                make_streamer_idle_when_not_moving,
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

#[derive(Default)]
pub struct MockSubscriberPlugin;

impl Plugin for MockSubscriberPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SubscriberMsg>();
        app.add_event::<Msg>();
        app.add_systems(
            Update,
            (
                replace_subscriber_tile,
                swim_to_streamer_to_speak.after(replace_subscriber_tile),
                speak_to_streamer_from_subscriber.after(swim_to_streamer_to_speak),
                subscriber_waits_to_leave_from_streamer.after(speak_to_streamer_from_subscriber),
                leave_from_streamer_from_subscriber.after(subscriber_waits_to_leave_from_streamer),
                return_subscriber_to_idle,
                follow_streamer_while_approaching_for_subscriber,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockFruitPlugin;

impl Plugin for MockFruitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                replace_fruit_tiles,
                make_fruit_fall.after(replace_fruit_tiles),
                make_fruit_dropped.after(make_fruit_fall),
                pathfind_streamer_to_fruit.after(make_fruit_dropped),
                respawn_fruit.after(pathfind_streamer_to_fruit),
            ),
        );
    }
}

#[derive(Default)]
pub struct MockCropPlugin;

impl Plugin for MockCropPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewSubscriber>();
        app.add_systems(
            Update,
            (
                replace_crop_tiles,
                grow_crops,
                pathfind_streamer_to_crops,
                pick_up_crops,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockPortraitPreferencePlugin;

impl Plugin for MockPortraitPreferencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PortraitPreferences::new(
            String::from(":memory:"),
            DEFAULT_SUBSCRIBER_SPRITE_IDX,
        ));
    }
}

#[derive(Default)]
pub struct MockEnvironmentAnimationsPlugin;

impl Plugin for MockEnvironmentAnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, replace_campfire_tile);
        app.add_systems(Update, insert_animation_information);
        app.add_systems(Update, make_streamer_face_campfire);
    }
}

/// Sets the time to move for any Entity to be instant
/// for testing purposes.
fn reduce_movement_times_to_zero(mut timer_query: Query<&mut MovementTimer, Added<MovementTimer>>) {
    for mut movement_timer in &mut timer_query {
        movement_timer.0 = Timer::new(Duration::from_secs(0), TimerMode::Repeating);
    }
}

/// Intercepts and sets the Wait Timer interval to 0 seconds for testing purposes.
pub fn reduce_wait_times_to_zero(
    mut waiting_timers: Query<&mut WaitToLeaveTimer, Added<WaitToLeaveTimer>>,
) {
    for mut waiting_timer in &mut waiting_timers {
        waiting_timer.0 = Timer::new(Duration::from_secs(0), TimerMode::Once);
    }
}

/// Sets each Typing Timer to zero to make testing
/// not dependent off of real-world time.
pub fn intercept_typing_timer(
    mut typing_timers: Query<&mut TypingSpeedTimer, Added<TypingSpeedTimer>>,
) {
    for mut typing_timer in &mut typing_timers {
        *typing_timer = TypingSpeedTimer(Timer::from_seconds(0.0, TimerMode::Repeating));
    }
}

/// Sets each Msg Waiting Timer to zero to make
/// testing not dependent off of real-world time.
pub fn intercept_msg_waiting_timer(
    mut message_waiting_timers: Query<&mut MsgWaitingTimer, Changed<MsgWaitingTimer>>,
) {
    for mut message_waiting_timer in &mut message_waiting_timers {
        *message_waiting_timer = MsgWaitingTimer(Timer::from_seconds(0.0, TimerMode::Once));
    }
}

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct GameWorld {
    pub app: App,
}

impl GameWorld {
    pub fn new() -> Self {
        let mut app = App::new();

        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.insert_state(GameState::InGame);
        app.add_plugins(MinimalPlugins);

        app.add_systems(Update, reduce_movement_times_to_zero);

        Self { app }
    }

    /// Advances the game n ticks.
    pub fn update(&mut self, num_ticks: usize) {
        for _i in 0..num_ticks {
            self.app.update();
        }
    }

    /// Returns the Component found within the game,
    /// or None otherwise.
    pub fn find<T>(&mut self) -> Option<&T>
    where
        T: Component,
    {
        self.app
            .world_mut()
            .query::<&T>()
            .get_single(&self.app.world())
            .ok()
    }

    /// Returns a list of Components found within the game
    pub fn find_all<T>(&mut self) -> Vec<&T>
    where
        T: Component,
    {
        self.app
            .world_mut()
            .query::<&T>()
            .iter(&self.app.world())
            .collect::<Vec<&T>>()
    }

    /// Returns the requested Component found associated
    /// with the other Component within the game, or None
    /// otherwise.
    pub fn find_with<T, U>(&mut self) -> Option<&T>
    where
        T: Component,
        U: Component,
    {
        self.app
            .world_mut()
            .query_filtered::<&T, With<U>>()
            .get_single(&self.app.world())
            .ok()
    }

    /// Returns true whenever there exists one or more of the
    /// requested Component within the game when a condition is met,
    /// or false otherwise.
    pub fn contains_when<T, U>(&mut self) -> bool
    where
        T: Component,
        U: QueryFilter,
    {
        self.app
            .world_mut()
            .query_filtered::<&T, U>()
            .iter(&self.app.world())
            .count()
            > 0
    }

    /// Sends an Event to all systems listening to it
    /// in the game.
    pub fn broadcast_event<T>(&mut self, event: T)
    where
        T: Event,
    {
        self.app.world_mut().send_event(event);
    }
}
