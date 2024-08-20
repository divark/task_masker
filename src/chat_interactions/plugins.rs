use bevy::prelude::*;

use crate::chat_interactions::twitch_chat_reader::*;

#[derive(Default)]
pub struct TwitchChatPlugin;

impl Plugin for TwitchChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Notification>();

        let twitch_msg_reader = TwitchMsgReader::connect_anonymously(String::from("divarktech"));

        app.insert_resource(twitch_msg_reader);
        app.add_systems(
            Update,
            (notify_all_about_twitch_msg, convert_notification_to_msg),
        );
    }
}
