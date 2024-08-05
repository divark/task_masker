mod mock_plugins;

use bevy::state::app::StatesPlugin;
use mock_plugins::{
    MockChatterPlugin, MockChattingPlugin, MockStreamerPlugin, MockSubscriberPlugin,
    MockTiledMapPlugin,
};

use bevy::prelude::*;
use cucumber::{given, then, when, World};

use task_masker::entities::MovementType;
use task_masker::map::plugins::TilePosEvent;
use task_masker::ui::chatting::*;
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
        MovementType::Walk,
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
        MovementType::Fly,
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
        MovementType::Swim,
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
    assert_eq!(msg.idx(), 5);
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

fn main() {
    futures::executor::block_on(GameWithChatUI::run("tests/feature-files/chatting.feature"));
}
