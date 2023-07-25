use bevy::prelude::*;

use crate::entities::MovementType;

use super::screens::SpeakerUI;

#[derive(Default, Event)]
pub struct Msg {
    speaker_name: String,
    msg: String,
    speaker_role: MovementType,
}

#[derive(Component, Deref, DerefMut)]
pub struct MsgIndex(usize);

#[derive(Component)]
pub struct MsgWaiting(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct TypingSpeedInterval(Timer);

pub fn setup_chatting_from_msg(
    mut chatting_fields: Query<
        (Entity, &mut UiImage, &mut Text),
        (
            With<SpeakerUI>,
            Without<TypingSpeedInterval>,
            Without<MsgWaiting>,
        ),
    >,
    mut msg_receiver: EventReader<Msg>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if chatting_fields.is_empty() {
        return;
    }

    let (ui_fields_entity, mut speaker_portrait, mut speaker_textbox) = chatting_fields
        .get_single_mut()
        .expect("Could not find Speaker UI elements.");

    if msg_receiver.is_empty() {
        return;
    }

    let recent_msg = msg_receiver
        .iter()
        .last()
        .expect("Should have a message pending.");

    let role_image = match &recent_msg.speaker_role {
        MovementType::Walk => UiImage::new(asset_server.load("caveman/portrait.png")),
        MovementType::Fly => UiImage::new(asset_server.load("chatters/portrait.png")),
        MovementType::Swim => UiImage::new(asset_server.load("subscribers/portrait.png")),
    };

    *speaker_portrait = role_image;
    speaker_textbox.sections[0].value = format!("{}:\n", recent_msg.speaker_name);

    for msg_character in recent_msg.msg.chars() {
        let untyped_character = TextSection::new(
            msg_character,
            TextStyle {
                color: Color::NONE,
                ..default()
            },
        );

        speaker_textbox.sections.push(untyped_character);
    }

    commands.entity(ui_fields_entity).insert((
        TypingSpeedInterval(Timer::from_seconds(0.1, TimerMode::Repeating)),
        MsgIndex(1),
    ));
}

pub fn teletype_current_message(
    mut chatting_fields: Query<(&mut Text, &mut MsgIndex, &mut TypingSpeedInterval)>,
    time: Res<Time>,
) {
    if chatting_fields.is_empty() {
        return;
    }

    let (mut speaker_msg, mut msg_index, mut typing_speed_timer) = chatting_fields
        .get_single_mut()
        .expect("Could not retrieve teletype UI information.");

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
