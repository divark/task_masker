mod mock_plugins;

use bevy::state::app::StatesPlugin;
use mock_plugins::{
    MockChatterPlugin, MockChattingPlugin, MockStreamerPlugin, MockSubscriberPlugin,
    MockTiledMapPlugin,
};

use bevy::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::chatter::ChatterStatus;
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

    /// Sends an Event to all systems listening to it
    /// in the game.
    pub fn broadcast_event<T>(&mut self, event: T)
    where
        T: Event,
    {
        self.app.world_mut().send_event(event);
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
        let msg_char = &textfield.sections[i];
        // All characters are invisible by default, which
        // we don't want to read, since that means that
        // these characters are not rendered on the screen.
        if msg_char.style.color != Color::BLACK {
            break;
        }

        msg_contents += &msg_char.value;
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
    world.update(1);
}

#[given("a Chatter is spawned on the map,")]
fn spawn_chatter(world: &mut GameWithChatUI) {
    world.app.add_plugins(MockChatterPlugin);
    world.update(1);
}

#[given("a Subscriber is spawned on the map,")]
fn spawn_subscriber(world: &mut GameWithChatUI) {
    world.app.add_plugins(MockSubscriberPlugin);
    world.update(1);
}

#[given("the Chatting interface exists,")]
fn spawn_chatting_ui(world: &mut GameWithChatUI) {
    world.app.add_plugins(MockChattingPlugin);
    world.update(1);
}

#[when("the Streamer sends a chat message,")]
fn streamer_sends_msg(world: &mut GameWithChatUI) {
    let streamer_msg = Msg::new(
        String::from("Caveman"),
        String::from("👍This is a test message to see if this works and types as expected."),
        GameEntityType::Walk,
    );

    world.broadcast_event::<Msg>(streamer_msg.clone());
    world.update(2);

    world.sent_msgs.push(streamer_msg);
}

#[when("the Chatter sends a chat message,")]
fn chatter_sends_msg(world: &mut GameWithChatUI) {
    let chatter_msg = Msg::new(
        String::from("Chatter"),
        String::from("Hello caveman!"),
        GameEntityType::Fly,
    );

    world.broadcast_event::<Msg>(chatter_msg.clone());
    world.update(2);

    world.sent_msgs.push(chatter_msg);
}

#[when("the Chatter sends a long chat message,")]
fn chatter_sends_long_msg(world: &mut GameWithChatUI) {
    let long_msg = String::from("So, if you're learning a subject of math for the first time, it's helpful to actually learn about the concepts behind it before going into the course, since you're otherwise being overloaded with a bunch of terminology. Doing it this way, it's important to do so with the angle of finding how it's important to your work, using analogies and metaphors to make the knowledge personal");

    let chatter_msg = Msg::new(String::from("Chatter"), long_msg, GameEntityType::Fly);

    world.broadcast_event::<Msg>(chatter_msg.clone());
    world.update(2);

    world.sent_msgs.push(chatter_msg);
}

#[when("the Subscriber sends a chat message,")]
fn subscriber_sends_msg(world: &mut GameWithChatUI) {
    let subscriber_msg = Msg::new(
        String::from("Subscriber"),
        String::from("'Ello caveman!"),
        GameEntityType::Swim,
    );

    world.broadcast_event::<Msg>(subscriber_msg.clone());
    world.update(2);

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

#[when("the Chatter is almost done speaking to the Streamer,")]
fn wait_until_chatter_near_end_of_speaking(world: &mut GameWithChatUI) {
    loop {
        world.update(1);

        let msg_index = world
            .find::<TypingMsg>()
            .expect(
                "wait_until_chatter_near_end_of_speaking: Could not find Typing Indicator type.",
            )
            .idx();
        if msg_index > 30 {
            break;
        }
    }
}

#[then("the Chatter should still be speaking.")]
fn chatter_should_still_be_speaking(world: &mut GameWithChatUI) {
    world.update(1);

    let msg_is_still_being_typed = !world
        .find::<TypingMsg>()
        .expect("chatter_should_still_be_speaking: Typing Indicator could not be found")
        .at_end();
    assert!(msg_is_still_being_typed);

    let expected_chatter_status = ChatterStatus::Speaking;
    let actual_chatter_status = world
        .find::<ChatterStatus>()
        .expect("chatter_should_still_be_speaking: Chatter status could not be found.");

    assert_eq!(expected_chatter_status, *actual_chatter_status);
}

#[then(
    regex = r"^the Chatting Queue should contain the (Streamer|Chatter|Subscriber)'s chat message."
)]
fn chatting_queue_has_streamer_msg(world: &mut GameWithChatUI) {
    world.update(1);

    let expected_msg_contents = world.sent_msgs.get(0).unwrap().clone();

    let pending_chat_messages = world
        .find::<MessageQueue>()
        .expect("chatting_queue_has_streamer_msg: Could not find Message Queue.");
    let next_chat_msg = pending_chat_messages.peek();
    assert!(next_chat_msg.is_some());

    let actual_chat_msg_contents = next_chat_msg.unwrap();
    assert_eq!(expected_msg_contents, *actual_chat_msg_contents);
}

#[then("the Chatting Queue should have the Streamer's chat message as the top priority.")]
fn chatting_queue_has_streamer_msg_top_priority(world: &mut GameWithChatUI) {
    world.update(1);

    // For testing purposes, the Streamer's message is the second one we've recorded
    // from earlier steps defined in this file.
    let expected_msg_contents = world.sent_msgs.get(1).unwrap().clone();

    let pending_chat_messages = world
        .find::<MessageQueue>()
        .expect("chatting_queue_has_streamer_msg: Could not find Message Queue.");
    let next_chat_msg = pending_chat_messages.peek();
    assert!(next_chat_msg.is_some());

    let actual_chat_msg_contents = next_chat_msg.unwrap();
    assert_eq!(expected_msg_contents, *actual_chat_msg_contents);
}

#[then(regex = r"there should be ([0-9]+) messages? in the Message Queue,")]
fn chatting_queue_has_n_messages(world: &mut GameWithChatUI, num_messages: String) {
    world.update(1);

    let chatting_queue = world
        .find::<MessageQueue>()
        .expect("chatting_queue_has_n_messages: Message Queue was not found.");

    let expected_num_messages = num_messages
        .parse::<usize>()
        .expect("chatting_queue_has_n_messages: Cannot convert number from then step.");
    let actual_num_messages = chatting_queue.len();
    assert_eq!(expected_num_messages, actual_num_messages);
}

#[then("the Chat UI should contain the first five characters typed from the Chat Message.")]
fn chatting_ui_contains_first_five_chars_from_msg(world: &mut GameWithChatUI) {
    world.update(1);

    let msg_txtfield = world.find_with::<Text, SpeakerChatBox>().expect(
        "chatting_ui_contains_first_five_chars_from_msg: Could not find Text from SpeakerChatBox.",
    );

    let expected_contents = String::from("👍This");
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
        .find_with::<Visibility, SpeakerUI>()
        .expect("chat_ui_should_be_hidden: Could not find Visibility for Chat UI.");

    assert_eq!(Visibility::Hidden, *ui_visibility);
}

fn main() {
    futures::executor::block_on(GameWithChatUI::run("tests/feature-files/chatting.feature"));
}
