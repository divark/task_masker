mod mock_plugins;

use crate::mock_plugins::{MockChattingPlugin, MockStreamerPlugin, MockTiledMapPlugin};

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

    pub expected_msg: Option<Msg>,
}

impl GameWithChatUI {
    pub fn new() -> Self {
        let mut app = App::new();

        app.init_state::<GameState>();
        app.insert_state(GameState::InGame);
        app.add_plugins(MinimalPlugins);

        Self {
            app,
            expected_msg: None,
        }
    }
}

#[given("a Streamer is spawned on the map,")]
fn spawn_streamer(world: &mut GameWithChatUI) {
    // A Streamer only exists in context of a map,
    // so we have to load a Tiled map as a prerequisite.
    world.app.add_plugins(MockTiledMapPlugin);
    world.app.update();

    // One way a Streamer reacts is by being told
    // where to go. The Mock plugin for the Streamer
    // has systems that depend on this fact, so we
    // include it here, despite it not being used
    // under these tests.
    world.app.add_event::<TilePosEvent>();
    world.app.add_plugins(MockStreamerPlugin);
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

    world.app.world.send_event::<Msg>(streamer_msg.clone());
    world.app.update();

    world.expected_msg = Some(streamer_msg);
}

#[then("the Chatting Queue should contain the Streamer's chat message.")]
fn chatting_queue_has_streamer_msg(world: &mut GameWithChatUI) {
    let pending_chat_messages = world
        .app
        .world
        .query::<&MessageQueue>()
        .single(&world.app.world);

    let next_chat_msg = pending_chat_messages.peek();
    assert!(next_chat_msg.is_some());

    let next_chat_msg_contents = next_chat_msg.unwrap();
    assert_eq!(world.expected_msg.clone().unwrap(), *next_chat_msg_contents);
}

fn main() {
    futures::executor::block_on(GameWithChatUI::run("tests/feature-files/chatting.feature"));
}
