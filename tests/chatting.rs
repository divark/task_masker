mod mock_plugins;

use bevy::state::app::StatesPlugin;
use mock_plugins::{
    MockChatterPlugin, MockChattingPlugin, MockStreamerPlugin, MockSubscriberPlugin,
    MockTiledMapPlugin,
};

use bevy::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::GameEntityType;
use task_masker::map::plugins::TilePosEvent;
use task_masker::ui::chatting::*;
use task_masker::ui::screens::{SpeakerChatBox, SpeakerUI};
use task_masker::GameState;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct GameWithChatUI {
    pub app: App,

    pub sent_msgs: Vec<Msg>,
}

impl GameWithChatUI {
    pub fn new() -> Self {
        let mut app = App::new();

        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.insert_state(GameState::InGame);
        app.add_plugins(MinimalPlugins);

        // A Streamer only exists in context of a map,
        // so we have to load a Tiled map as a prerequisite.
        app.add_plugins(MockTiledMapPlugin);
        app.add_event::<TilePosEvent>();

        app.add_systems(Update, intercept_typing_timer);
        app.update();

        Self {
            app,
            sent_msgs: Vec::new(),
        }
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
}

/// Sets each Typing Timer to zero to make testing
/// not dependent off of real-world time.
fn intercept_typing_timer(
    mut typing_timers: Query<&mut TypingSpeedTimer, Added<TypingSpeedTimer>>,
) {
    for mut typing_timer in &mut typing_timers {
        *typing_timer = TypingSpeedTimer(Timer::from_seconds(0.0, TimerMode::Repeating));
    }
}

/// Sets each Waiting Timer to zero to make testing
/// not dependent off of real-world time.
fn intercept_msg_waiting_timer(mut waiting_timers: Query<&mut MsgWaitingTimer>) {
    for mut waiting_timer in &mut waiting_timers {
        *waiting_timer = MsgWaitingTimer(Timer::from_seconds(0.0, TimerMode::Once));
    }
}

/// Returns the first n characters found from some
/// Text field as a String.
fn read_first_n(textfield: &Text, amount: usize) -> String {
    let mut msg_contents = String::new();

    for i in 1..=amount {
        msg_contents += &textfield.sections[i].value;
    }

    msg_contents
}

#[given("a Streamer is spawned on the map,")]
fn spawn_streamer(world: &mut GameWithChatUI) {
    // One way a Streamer reacts is by being told
    // where to go. The Mock plugin for the Streamer
    // has systems that depend on this fact, so we
    // include it here, despite it not being used
    // under these tests.
    world.app.add_plugins(MockStreamerPlugin);
    world.app.update();
}

#[given("a Chatter is spawned on the map,")]
fn spawn_chatter(world: &mut GameWithChatUI) {
    world.app.add_plugins(MockChatterPlugin);
    world.app.update();
}

#[given("a Subscriber is spawned on the map,")]
fn spawn_subscriber(world: &mut GameWithChatUI) {
    world.app.add_plugins(MockSubscriberPlugin);
    world.app.update();
}

#[given("the Chatting interface exists,")]
fn spawn_chatting_ui(world: &mut GameWithChatUI) {
    world.app.add_plugins(MockChattingPlugin);
    world.app.update();
}

#[when("the Streamer sends a chat message,")]
fn streamer_sends_msg(world: &mut GameWithChatUI) {
    let streamer_msg = Msg::new(
        String::from("Caveman"),
        String::from("This is a test message to see if this works and types as expected."),
        GameEntityType::Walk,
    );

    world
        .app
        .world_mut()
        .send_event::<Msg>(streamer_msg.clone());
    world.app.update();
    world.app.update();

    world.sent_msgs.push(streamer_msg);
}

#[when("the Chatter sends a chat message,")]
fn chatter_sends_msg(world: &mut GameWithChatUI) {
    let chatter_msg = Msg::new(
        String::from("Chatter"),
        String::from("Hello caveman!"),
        GameEntityType::Fly,
    );

    world.app.world_mut().send_event::<Msg>(chatter_msg.clone());
    world.app.update();
    world.app.update();

    world.sent_msgs.push(chatter_msg);
}

#[when("the Subscriber sends a chat message,")]
fn subscriber_sends_msg(world: &mut GameWithChatUI) {
    let subscriber_msg = Msg::new(
        String::from("Subscriber"),
        String::from("'Ello caveman!"),
        GameEntityType::Swim,
    );

    world
        .app
        .world_mut()
        .send_event::<Msg>(subscriber_msg.clone());
    world.app.update();
    world.app.update();

    world.sent_msgs.push(subscriber_msg);
}

#[when("the first five characters of the chat message has been read,")]
fn types_five_characters_from_msg(world: &mut GameWithChatUI) {
    world.update(5);

    let msg = world
        .find::<TypingMsg>()
        .expect("types_five_characters_from_msg: Could not find TypingMsg Component.");
    assert!(msg.idx() >= 5);
}

#[when("the chat message has been fully read,")]
fn read_whole_chat_msg(world: &mut GameWithChatUI) {
    loop {
        world.update(1);

        let msg_being_typed = world
            .find::<TypingMsg>()
            .expect("read_whole_chat_msg: Could not find current chat msg being typed.");

        if msg_being_typed.at_end() {
            break;
        }
    }
}

#[when("the wait time is up,")]
fn wait_until_wait_time_is_up(world: &mut GameWithChatUI) {
    world
        .app
        .add_systems(Update, intercept_msg_waiting_timer.run_if(run_once()));

    loop {
        world.update(1);

        let found_waiting_timer = world.find::<MsgWaitingTimer>();
        if found_waiting_timer.is_none() {
            break;
        }
    }
}

#[then(
    regex = r"^the Chatting Queue should contain the (Streamer|Chatter|Subscriber)'s chat message."
)]
fn chatting_queue_has_streamer_msg(world: &mut GameWithChatUI) {
    world.app.update();

    let pending_chat_messages = world
        .app
        .world_mut()
        .query::<&MessageQueue>()
        .single(&world.app.world());

    let next_chat_msg = pending_chat_messages.peek();
    assert!(next_chat_msg.is_some());

    let next_chat_msg_contents = next_chat_msg.unwrap();
    assert_eq!(*world.sent_msgs.get(0).unwrap(), *next_chat_msg_contents);
}

#[then("the Chatting Queue should have the Streamer's chat message as the top priority.")]
fn chatting_queue_has_streamer_msg_top_priority(world: &mut GameWithChatUI) {
    world.app.update();

    let pending_chat_messages = world
        .app
        .world_mut()
        .query::<&MessageQueue>()
        .single(&world.app.world());

    let next_chat_msg = pending_chat_messages.peek();
    assert!(next_chat_msg.is_some());

    let next_chat_msg_contents = next_chat_msg.unwrap();
    assert_eq!(*world.sent_msgs.get(1).unwrap(), *next_chat_msg_contents);
}

#[then("the Chat UI should contain the first five characters typed from the Chat Message.")]
fn chatting_ui_contains_first_five_chars_from_msg(world: &mut GameWithChatUI) {
    world.update(1);

    let msg_txtfield = world
        .app
        .world_mut()
        .query_filtered::<&Text, With<SpeakerChatBox>>()
        .single(&world.app.world());

    let expected_contents = String::from("This ");
    let msg_contents = read_first_n(&msg_txtfield, 5);

    assert_eq!(expected_contents, msg_contents);
}

#[then("the Chat Message should no longer be present,")]
fn chatting_msg_should_be_gone(world: &mut GameWithChatUI) {
    world.update(1);

    let found_msg = world.find::<TypingMsg>();
    assert!(found_msg.is_none());
}

#[then("the Chat UI should be hidden.")]
fn chat_ui_should_be_hidden(world: &mut GameWithChatUI) {
    world.update(1);

    let ui_visibility = world
        .app
        .world_mut()
        .query_filtered::<&Visibility, With<SpeakerUI>>()
        .get_single(&world.app.world())
        .expect("chat_ui_should_be_hidden: Could not find Visibility for Chat UI.");

    assert_eq!(Visibility::Hidden, *ui_visibility);
}

fn main() {
    futures::executor::block_on(GameWithChatUI::run("tests/feature-files/chatting.feature"));
}
