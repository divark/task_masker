use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bevy::prelude::*;

use super::screens::{SpeakerChatBox, SpeakerPortrait, SpeakerUI};
use crate::entities::MovementType;

#[derive(Component, PartialEq)]
pub enum ChattingStatus {
    Idle,
    Speaking(MovementType),
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
    pub speaker_role: MovementType,
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
    pub fn new(speaker_name: String, speaker_msg: String, speaker_role: MovementType) -> Self {
        let speaker_priority = match speaker_role {
            MovementType::Walk => MsgPriority::High,
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
pub struct MsgIndex(usize);

#[derive(Component, Deref, DerefMut)]
pub struct MsgLen(usize);

#[derive(Component, Deref, DerefMut)]
pub struct MsgWaiting(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct TypingSpeedInterval(Timer);

#[derive(Bundle)]
pub struct Chatting {
    pending_messages: MessageQueue,
    typing_speed: TypingSpeedInterval,
    msg_waiting: MsgWaiting,
    msg_character_idx: MsgIndex,
    msg_len: MsgLen,
    status: ChattingStatus,
}

#[derive(Component)]
pub struct TypingMsg {
    msg: String,

    msg_idx: usize,
}

impl TypingMsg {
    pub fn new(msg_contents: String) -> Self {
        Self {
            msg: msg_contents,
            msg_idx: 0,
        }
    }

    /// Returns the number of times this message has been
    /// told to advance to the next character.
    pub fn num_typed_chars(&self) -> usize {
        self.msg_idx
    }
}

pub fn insert_chatting_information(
    chatting_fields: Query<Entity, (With<SpeakerChatBox>, Without<TypingSpeedInterval>)>,
    mut commands: Commands,
) {
    if chatting_fields.is_empty() {
        return;
    }

    let ui_fields_entity = chatting_fields
        .get_single()
        .expect("Chatting UI should exist by now.");

    let mut typing_speed_timer =
        TypingSpeedInterval(Timer::from_seconds(0.1, TimerMode::Repeating));
    typing_speed_timer.pause();
    let mut msg_waiting_timer = MsgWaiting(Timer::from_seconds(15.0, TimerMode::Repeating));
    msg_waiting_timer.pause();

    let msg_character_idx = MsgIndex(0);
    let msg_len = MsgLen(0);

    commands.entity(ui_fields_entity).insert(Chatting {
        pending_messages: MessageQueue(BinaryHeap::new()),
        typing_speed: typing_speed_timer,
        msg_waiting: msg_waiting_timer,
        msg_character_idx,
        msg_len,
        status: ChattingStatus::Idle,
    });
}

/// Adds a recently captured message into the pending messages queue.
pub fn add_msg_to_pending(
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

pub fn setup_chatting_from_msg(
    mut chatting_ui_section: Query<&mut Visibility, With<SpeakerUI>>,
    mut chatting_fields: Query<(&mut UiImage, &mut TextureAtlas), With<SpeakerPortrait>>,
    mut msg_fields: Query<
        (
            &mut Text,
            &mut MsgIndex,
            &mut MsgLen,
            &mut TypingSpeedInterval,
            &mut ChattingStatus,
            &mut MessageQueue,
        ),
        With<SpeakerChatBox>,
    >,
    chatting_entities: Query<
        (&TextureAtlas, &Handle<Image>, &MovementType),
        Without<SpeakerPortrait>,
    >,
) {
    if chatting_entities.is_empty() {
        return;
    }

    if chatting_ui_section.is_empty() {
        return;
    }

    if msg_fields.is_empty() {
        return;
    }

    let (mut speaker_image, mut speaker_portrait_atlas) = chatting_fields
        .get_single_mut()
        .expect("Could not find Speaker UI elements.");

    let (
        mut speaker_textbox,
        mut msg_index,
        mut msg_len,
        mut typing_speed_timer,
        mut chatting_status,
        mut pending_msgs_queue,
    ) = msg_fields
        .get_single_mut()
        .expect("Msg elements should be attached by now.");

    if pending_msgs_queue.is_empty() || msg_len.0 != 0 {
        return;
    }

    let recent_msg = pending_msgs_queue
        .pop()
        .expect("Should have a message pending.");

    let (role_image, role_atlas) = match &recent_msg.speaker_role {
        MovementType::Walk => {
            let streamer_texture_entry = chatting_entities
                .iter()
                .find(|entity_texture_info| *entity_texture_info.2 == MovementType::Walk)
                .expect("setup_chatting_from_msg: Could not find Streamer's Texture Atlas.");

            let streamer_texture_atlas = streamer_texture_entry.0.clone();
            let streamer_image_handle = streamer_texture_entry.1.clone();

            let streamer_image = UiImage::new(streamer_image_handle);

            (streamer_image, streamer_texture_atlas)
        }
        MovementType::Fly => {
            let chatter_texture_entry = chatting_entities
                .iter()
                .find(|entity_texture_info| *entity_texture_info.2 == MovementType::Fly)
                .expect("setup_chatting_from_msg: Could not find Chatter's Texture Atlas.");

            let chatter_texture_atlas = chatter_texture_entry.0.clone();
            let chatter_image_handle = chatter_texture_entry.1.clone();

            let chatter_image = UiImage::new(chatter_image_handle);

            (chatter_image, chatter_texture_atlas)
        }
        MovementType::Swim => {
            let subscriber_texture_entry = chatting_entities
                .iter()
                .find(|entity_texture_info| *entity_texture_info.2 == MovementType::Swim)
                .expect("setup_chatting_from_msg: Could not find Subscriber's Texture Atlas.");

            let subscriber_texture_atlas = subscriber_texture_entry.0.clone();
            let subscriber_image_handle = subscriber_texture_entry.1.clone();

            let subscriber_image = UiImage::new(subscriber_image_handle);

            (subscriber_image, subscriber_texture_atlas)
        }
    };

    *speaker_portrait_atlas = role_atlas;
    *speaker_image = role_image;

    speaker_textbox.sections[0].value = String::new();
    speaker_textbox.sections.drain(1..);

    speaker_textbox.sections[0].value = format!("{}:\n", recent_msg.speaker_name);
    speaker_textbox.sections[0].style.font_size = 32.0;
    speaker_textbox.sections[0].style.color = Color::BLACK;

    for msg_character in recent_msg.msg.chars() {
        let untyped_character = TextSection::new(
            msg_character,
            TextStyle {
                color: Color::NONE,
                font_size: 28.0,
                ..default()
            },
        );

        speaker_textbox.sections.push(untyped_character);
    }

    msg_index.0 = 1;
    msg_len.0 = recent_msg.msg.len();

    let mut speaker_ui_visibility = chatting_ui_section
        .get_single_mut()
        .expect("Speaker UI should exist by now.");
    *speaker_ui_visibility = Visibility::Visible;

    *chatting_status = ChattingStatus::Speaking(recent_msg.speaker_role);

    typing_speed_timer.reset();
    typing_speed_timer.unpause();
}

pub fn teletype_current_message(
    mut msg_fields: Query<
        (&mut Text, &mut MsgIndex, &mut TypingSpeedInterval),
        With<SpeakerChatBox>,
    >,
    time: Res<Time>,
) {
    if msg_fields.is_empty() {
        return;
    }

    let (mut speaker_msg, mut msg_index, mut typing_speed_timer) = msg_fields
        .get_single_mut()
        .expect("Msg elements should be attached by now.");

    typing_speed_timer.tick(time.delta());
    if !typing_speed_timer.0.just_finished() {
        return;
    }

    let msg_character = speaker_msg
        .sections
        .get_mut(msg_index.0)
        .expect("Could not find text section in msg.");
    msg_character.style.color = Color::BLACK;

    msg_index.0 += 1;
}

/// Spawns a noise for each visible character just revealed.
pub fn play_typing_noise(
    msg_fields: Query<(&Text, &MsgIndex, &MsgLen), Changed<MsgIndex>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (speaker_msg, msg_index, msg_len) in &msg_fields {
        if msg_len.0 == 0 {
            continue;
        }

        let msg_character = speaker_msg
            .sections
            .get(msg_index.0)
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

pub fn activate_waiting_timer(
    mut chatting_information: Query<
        (
            &MsgIndex,
            &MsgLen,
            &mut TypingSpeedInterval,
            &mut MsgWaiting,
        ),
        Changed<MsgIndex>,
    >,
) {
    if chatting_information.is_empty() {
        return;
    }

    let (msg_index, msg_len, mut typing_speed_timer, mut msg_waiting_timer) = chatting_information
        .get_single_mut()
        .expect("Chatting information should exist.");

    if msg_index.0 != msg_len.0 && msg_len.0 != 0 {
        return;
    }

    typing_speed_timer.pause();
    typing_speed_timer.reset();

    msg_waiting_timer.reset();
    msg_waiting_timer.unpause();
}

pub fn clear_current_msg_on_time_up(
    mut msg_fields: Query<
        (&mut MsgLen, &mut MsgWaiting, &mut ChattingStatus),
        With<SpeakerChatBox>,
    >,
    time: Res<Time>,
) {
    if msg_fields.is_empty() {
        return;
    }

    let (mut msg_len, mut msg_waiting_timer, mut chatting_status) = msg_fields
        .get_single_mut()
        .expect("Waiting timer should exist with the UI by now.");

    if msg_waiting_timer.paused() {
        return;
    }

    msg_waiting_timer.tick(time.delta());
    if !msg_waiting_timer.just_finished() {
        return;
    }

    msg_waiting_timer.pause();
    msg_waiting_timer.reset();

    msg_len.0 = 0;
    *chatting_status = ChattingStatus::Idle;
}

pub fn hide_chatting_ui(
    msg_fields: Query<&MsgLen, Changed<MsgLen>>,
    mut speaker_ui_fields: Query<&mut Visibility, With<SpeakerUI>>,
) {
    if msg_fields.is_empty() || speaker_ui_fields.is_empty() {
        return;
    }

    let msg_len = msg_fields
        .get_single()
        .expect("Msg Len should be populated by now.");
    if msg_len.0 != 0 {
        return;
    }

    let mut speaker_ui_visibility = speaker_ui_fields
        .get_single_mut()
        .expect("Speaker UI should exist by now.");

    *speaker_ui_visibility = Visibility::Hidden;
}
