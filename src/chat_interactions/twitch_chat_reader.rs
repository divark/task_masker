use bevy::prelude::*;

use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

/// A Message found from Twitch that has not been parsed
/// yet.
#[derive(Event)]
pub struct Notification {
    msg: ServerMessage,
}

impl Notification {
    pub fn new(chat_msg: ServerMessage) -> Self {
        Self { msg: chat_msg }
    }
}

/// An interface for reading Twitch Messages from some
/// channel in real-time.
#[derive(Resource)]
pub struct TwitchMsgReader {
    msg_reader: UnboundedReceiver<ServerMessage>,
}

impl TwitchMsgReader {
    /// Creates a MsgReader listening to the specified
    /// Twitch channel name without logging in.
    pub fn connect_anonymously(channel_name: String) -> Self {
        let config = ClientConfig::default();
        let (incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        client
            .join(channel_name)
            .expect("TwitchMsgReader::new: Could not join Twitch channel");

        Self {
            msg_reader: incoming_messages,
        }
    }

    /// Returns some Notification if one was found from the Twitch
    /// chat, or None otherwise.
    pub fn read(&mut self) -> Option<Notification> {
        let msg_status = self.msg_reader.try_recv();
        if msg_status.is_err() {
            return None;
        }

        let msg_contents = msg_status.unwrap();
        Some(Notification::new(msg_contents))
    }
}

/// Broadcasts all recently found Twitch messages as
/// Notifications
pub fn notify_all_about_twitch_msg(
    mut twitch_msg_reader: ResMut<TwitchMsgReader>,
    mut notification_broadcaster: EventWriter<Notification>,
) {
    while let Some(notification) = twitch_msg_reader.read() {
        notification_broadcaster.send(notification);
    }
}

// TODO: Write systems that create Msgs depending on whether
// the notification is a message from the streamer, chatter,
// or subscriber.
