use bevy::prelude::*;

use crate::entities::MovementType;

use super::screens::{SpeakerChatBox, SpeakerPortrait, SpeakerUI};

#[derive(Default, Event)]
pub struct Msg {
    speaker_name: String,
    msg: String,
    speaker_role: MovementType,
}

#[derive(Component, Deref, DerefMut)]
pub struct MsgIndex(usize);

#[derive(Component, Deref, DerefMut)]
pub struct MsgLen(usize);

#[derive(Component, Deref, DerefMut)]
pub struct MsgWaiting(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct TypingSpeedInterval(Timer);

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

    commands.entity(ui_fields_entity).insert((
        typing_speed_timer,
        msg_waiting_timer,
        msg_character_idx,
        msg_len,
    ));
}

pub fn setup_chatting_from_msg(
    mut chatting_ui_section: Query<&mut Visibility, With<SpeakerUI>>,
    mut chatting_fields: Query<&mut UiImage, With<SpeakerPortrait>>,
    mut msg_fields: Query<
        (
            &mut Text,
            &mut MsgIndex,
            &mut MsgLen,
            &mut TypingSpeedInterval,
        ),
        With<SpeakerChatBox>,
    >,
    mut msg_receiver: EventReader<Msg>,
    asset_server: Res<AssetServer>,
) {
    if msg_receiver.is_empty() || chatting_ui_section.is_empty() {
        return;
    }

    let mut speaker_portrait = chatting_fields
        .get_single_mut()
        .expect("Could not find Speaker UI elements.");

    let (mut speaker_textbox, mut msg_index, mut msg_len, mut typing_speed_timer) = msg_fields
        .get_single_mut()
        .expect("Msg elements should be attached by now.");

    let recent_msg = msg_receiver
        .read()
        .next()
        .expect("Should have a message pending.");

    let role_image = match &recent_msg.speaker_role {
        MovementType::Walk => UiImage::new(asset_server.load("caveman/portrait.png")),
        MovementType::Fly => UiImage::new(asset_server.load("chatters/portrait.png")),
        MovementType::Swim => UiImage::new(asset_server.load("subscribers/portrait.png")),
    };

    *speaker_portrait = role_image;

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

    typing_speed_timer.reset();
    typing_speed_timer.unpause();
}

pub fn teletype_current_message(
    mut msg_fields: Query<
        (&mut Text, &mut MsgIndex, &mut TypingSpeedInterval),
        With<SpeakerChatBox>,
    >,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
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

    if msg_character.value == " " {
        return;
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
    mut msg_fields: Query<(&mut MsgLen, &mut MsgWaiting), With<SpeakerChatBox>>,
    time: Res<Time>,
) {
    if msg_fields.is_empty() {
        return;
    }

    let (mut msg_len, mut msg_waiting_timer) = msg_fields
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

pub fn test_streamer_msg(keyboard_input: Res<Input<KeyCode>>, mut msg_writer: EventWriter<Msg>) {
    if !keyboard_input.just_pressed(KeyCode::Q) {
        return;
    }

    let streamer_msg = Msg {
        speaker_name: "Caveman".to_string(),
        msg: "This is a test message to see if this works and types as expected.".to_string(),
        speaker_role: MovementType::Walk,
    };

    msg_writer.send(streamer_msg);
}
