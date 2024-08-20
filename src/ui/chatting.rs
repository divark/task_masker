use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bevy::prelude::*;

use super::screens::{SpeakerChatBox, SpeakerPortrait, SpeakerUI};
use crate::entities::GameEntityType;

#[derive(Component, PartialEq)]
pub enum ChattingStatus {
    Idle,
    Speaking(GameEntityType),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum MsgPriority {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Default, Event, Eq, PartialEq, Clone, Debug)]
pub struct Msg {
    pub speaker_name: String,
    pub msg: String,
    pub speaker_role: GameEntityType,
    speaker_priority: MsgPriority,
}

impl Ord for Msg {
    fn cmp(&self, other: &Self) -> Ordering {
        self.speaker_priority.cmp(&other.speaker_priority)
    }
}

impl PartialOrd for Msg {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Msg {
    pub fn new(speaker_name: String, speaker_msg: String, speaker_role: GameEntityType) -> Self {
        let speaker_priority = match speaker_role {
            GameEntityType::Walk => MsgPriority::High,
            _ => MsgPriority::Low,
        };

        Msg {
            speaker_name,
            msg: speaker_msg,
            speaker_role,
            speaker_priority,
        }
    }
}

/// A Priority Queue that ensures that a Streamer's messages
/// are always seen right away.
#[derive(Component, Deref, DerefMut)]
pub struct MessageQueue(BinaryHeap<Msg>);

#[derive(Component, Deref, DerefMut)]
pub struct MsgWaitingTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct TypingSpeedTimer(pub Timer);

#[derive(Bundle)]
pub struct Chatting {
    pending_messages: MessageQueue,
    status: ChattingStatus,
}

#[derive(Component)]
pub struct TypingMsg {
    msg: Msg,

    msg_idx: usize,
}

impl TypingMsg {
    pub fn new(msg_contents: Msg) -> Self {
        Self {
            msg: msg_contents,
            msg_idx: 0,
        }
    }

    /// Returns the Entity's type for the currently
    /// loaded message.
    pub fn speaker_role(&self) -> GameEntityType {
        self.msg.speaker_role
    }

    /// Returns whether the read message is at the last
    /// character or not.
    pub fn at_end(&self) -> bool {
        self.msg_idx == self.msg.msg.len()
    }

    /// Returns the index of the current character within
    /// the message.
    pub fn idx(&self) -> usize {
        self.msg_idx
    }

    /// Adjusts the index to point to the next character within
    /// the message.
    pub fn to_next_char(&mut self) {
        let new_idx = self.msg_idx + 1;

        if new_idx > self.msg.msg.len() {
            return;
        }

        self.msg_idx = new_idx;
    }
}

pub fn insert_chatting_information(
    mut msg_visibility_entry: Query<&mut Visibility, With<SpeakerUI>>,
    chatting_fields: Query<Entity, (With<SpeakerChatBox>, Without<MessageQueue>)>,
    mut commands: Commands,
) {
    if chatting_fields.is_empty() || msg_visibility_entry.is_empty() {
        return;
    }

    let ui_fields_entity = chatting_fields
        .get_single()
        .expect("Chatting UI should exist by now.");

    let mut msg_ui_visibility = msg_visibility_entry.single_mut();
    *msg_ui_visibility = Visibility::Hidden;

    commands.entity(ui_fields_entity).insert(Chatting {
        pending_messages: MessageQueue(BinaryHeap::new()),
        status: ChattingStatus::Idle,
    });
}

/// Adds a recently captured message into the pending messages queue.
pub fn load_msg_into_queue(
    mut message_queue_query: Query<&mut MessageQueue>,
    mut msg_receiver: EventReader<Msg>,
) {
    if message_queue_query.is_empty() {
        return;
    }

    let mut pending_messages = message_queue_query.single_mut();
    for received_msg in msg_receiver.read() {
        pending_messages.push(received_msg.clone());
    }
}

/// Populates the contents of the next message in the queue
/// into the Message UI Field.
pub fn load_queued_msg_into_textfield(
    mut msg_visibility_entry: Query<&mut Visibility, With<SpeakerUI>>,
    message_queue_entry: Query<&MessageQueue>,
    mut msg_fields: Query<
        (Entity, &mut Text, &mut ChattingStatus),
        (With<SpeakerChatBox>, Without<TypingMsg>),
    >,
    mut commands: Commands,
) {
    if message_queue_entry.is_empty() || msg_fields.is_empty() || msg_visibility_entry.is_empty() {
        return;
    }

    let (msg_entities, mut msg_textfield, mut chatting_status) = msg_fields.single_mut();
    let pending_msgs = message_queue_entry.single();

    if pending_msgs.is_empty() {
        return;
    }

    let recent_msg = pending_msgs.peek().unwrap();

    msg_textfield.sections.drain(1..);

    msg_textfield.sections[0].value = format!("{}:\n", recent_msg.speaker_name);
    msg_textfield.sections[0].style.font_size = 32.0;
    msg_textfield.sections[0].style.color = Color::BLACK;

    for msg_character in recent_msg.msg.chars() {
        let untyped_character = TextSection::new(
            msg_character,
            TextStyle {
                color: Color::NONE,
                font_size: 28.0,
                ..default()
            },
        );

        msg_textfield.sections.push(untyped_character);
    }

    let mut msg_ui_visibility = msg_visibility_entry.single_mut();
    *msg_ui_visibility = Visibility::Visible;
    *chatting_status = ChattingStatus::Speaking(recent_msg.speaker_role);

    let typing_speed_timer = TypingSpeedTimer(Timer::from_seconds(0.1, TimerMode::Repeating));
    commands
        .entity(msg_entities)
        .insert((TypingMsg::new(recent_msg.clone()), typing_speed_timer));
}

/// Loads the Speaker Portrait based on the currently
/// loaded message.
pub fn load_portrait_from_msg(
    msg_fields: Query<&TypingMsg, Added<TypingMsg>>,
    mut speaker_portrait: Query<(&mut UiImage, &mut TextureAtlas), With<SpeakerPortrait>>,
    chatting_portraits: Query<
        (&TextureAtlas, &Handle<Image>, &GameEntityType),
        Without<SpeakerPortrait>,
    >,
) {
    if msg_fields.is_empty() || speaker_portrait.is_empty() || chatting_portraits.is_empty() {
        return;
    }

    let (mut speaker_image, mut speaker_texture_atlas) = speaker_portrait.single_mut();
    let current_msg = msg_fields.single();

    let (role_image, role_atlas) = match &current_msg.speaker_role() {
        GameEntityType::Walk => {
            let streamer_texture_entry = chatting_portraits
                .iter()
                .find(|entity_texture_info| *entity_texture_info.2 == GameEntityType::Walk)
                .expect("setup_chatting_from_msg: Could not find Streamer's Texture Atlas.");

            let streamer_texture_atlas = streamer_texture_entry.0.clone();
            let streamer_image_handle = streamer_texture_entry.1.clone();

            let streamer_image = UiImage::new(streamer_image_handle);

            (streamer_image, streamer_texture_atlas)
        }
        GameEntityType::Fly => {
            let chatter_texture_entry = chatting_portraits
                .iter()
                .find(|entity_texture_info| *entity_texture_info.2 == GameEntityType::Fly)
                .expect("setup_chatting_from_msg: Could not find Chatter's Texture Atlas.");

            let chatter_texture_atlas = chatter_texture_entry.0.clone();
            let chatter_image_handle = chatter_texture_entry.1.clone();

            let chatter_image = UiImage::new(chatter_image_handle);

            (chatter_image, chatter_texture_atlas)
        }
        GameEntityType::Swim => {
            let subscriber_texture_entry = chatting_portraits
                .iter()
                .find(|entity_texture_info| *entity_texture_info.2 == GameEntityType::Swim)
                .expect("setup_chatting_from_msg: Could not find Subscriber's Texture Atlas.");

            let subscriber_texture_atlas = subscriber_texture_entry.0.clone();
            let subscriber_image_handle = subscriber_texture_entry.1.clone();

            let subscriber_image = UiImage::new(subscriber_image_handle);

            (subscriber_image, subscriber_texture_atlas)
        }
        _ => panic!("load_portrait_from_msg: Unsupported role detected."),
    };

    *speaker_image = role_image;
    *speaker_texture_atlas = role_atlas;
}

pub fn teletype_current_message(
    mut msg_fields: Query<(&mut Text, &mut TypingMsg, &mut TypingSpeedTimer), With<SpeakerChatBox>>,
    time: Res<Time>,
) {
    if msg_fields.is_empty() {
        return;
    }

    let (mut speaker_msg, mut typing_msg, mut typing_speed_timer) = msg_fields
        .get_single_mut()
        .expect("Msg elements should be attached by now.");

    typing_speed_timer.tick(time.delta());
    if !typing_speed_timer.0.just_finished() {
        return;
    }

    if typing_msg.at_end() {
        return;
    }

    let msg_character = speaker_msg
        .sections
        // The 2nd section is what actually holds
        // the chat message, so we have to shift it
        // accordingly.
        .get_mut(typing_msg.idx() + 1)
        .expect("Could not find text section in msg.");
    msg_character.style.color = Color::BLACK;

    typing_msg.to_next_char();
}

/// Spawns a noise for each visible character just revealed.
pub fn play_typing_noise(
    msg_fields: Query<(&Text, &TypingMsg), Changed<TypingMsg>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (speaker_msg, typing_msg) in &msg_fields {
        let msg_character = speaker_msg
            .sections
            .get(typing_msg.idx())
            .expect("play_typing_noise: Could not find text section in msg.");

        if msg_character.value == " " {
            continue;
        }

        let typing_noise = AudioBundle {
            source: asset_server.load("ui/balloon-boop.wav"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..default()
            },
        };

        commands.spawn(typing_noise);
    }
}

/// Begins to wait until the Chatting UI will clear itself once a message
/// has been fully typed.
pub fn activate_waiting_timer(
    mut chatting_information: Query<(Entity, &TypingMsg), Changed<TypingMsg>>,
    mut commands: Commands,
) {
    if chatting_information.is_empty() {
        return;
    }

    let (chatting_ui_entities, typing_msg) = chatting_information
        .get_single_mut()
        .expect("Chatting information should exist.");

    if !typing_msg.at_end() {
        return;
    }

    commands
        .entity(chatting_ui_entities)
        .remove::<TypingSpeedTimer>();

    let msg_waiting_timer = MsgWaitingTimer(Timer::from_seconds(5.0, TimerMode::Once));
    commands
        .entity(chatting_ui_entities)
        .insert(msg_waiting_timer);
}

/// Removes the most recent message, and hides the Chat UI.
pub fn unload_msg_on_timeup(
    mut message_queue_entry: Query<&mut MessageQueue>,
    mut msg_visibility_entry: Query<&mut Visibility, With<SpeakerUI>>,
    mut msg_fields: Query<
        (Entity, &mut MsgWaitingTimer, &mut ChattingStatus),
        With<SpeakerChatBox>,
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    if msg_fields.is_empty() || msg_visibility_entry.is_empty() || message_queue_entry.is_empty() {
        return;
    }

    let (chatting_ui_entities, mut msg_waiting_timer, mut chatting_status) = msg_fields
        .get_single_mut()
        .expect("Waiting timer should exist with the UI by now.");

    msg_waiting_timer.tick(time.delta());
    if !msg_waiting_timer.just_finished() {
        return;
    }

    let mut msg_ui_visibility = msg_visibility_entry.single_mut();
    *msg_ui_visibility = Visibility::Hidden;

    let mut pending_msgs = message_queue_entry.single_mut();
    pending_msgs.pop();

    commands
        .entity(chatting_ui_entities)
        .remove::<MsgWaitingTimer>();
    commands.entity(chatting_ui_entities).remove::<TypingMsg>();
    *chatting_status = ChattingStatus::Idle;
}
