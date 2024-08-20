use bevy::prelude::*;

use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{ServerMessage, ServerMessage::Privmsg};
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

use crate::entities::chatter::ChatMsg;

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

    /// Converts the contents of the Notification into
    /// a Msg if possible, or returns None otherwise.
    pub fn as_msg(&self) -> Option<ChatMsg> {
        if let Privmsg(current_msg) = &self.msg {
            let speaker_name = current_msg.sender.name.clone();
            let speaker_msg = current_msg.message_text.clone();

            Some(ChatMsg {
                name: speaker_name,
                msg: speaker_msg,
            })
        } else {
            None
        }
    }
}

/// An interface for reading Twitch Messages from some
/// channel in real-time.
#[derive(Resource)]
pub struct TwitchMsgReader {
    _rt: Runtime,
    _background_task: JoinHandle<()>,
    message_receiver: Receiver<ServerMessage>,
}

impl TwitchMsgReader {
    /// Creates a MsgReader listening to the specified
    /// Twitch channel name without logging in.
    pub fn connect_anonymously(channel_name: String) -> Self {
        let _rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (message_writer, message_reader) = mpsc::channel(100);
        let _background_task = _rt.spawn(async move {
            let config = ClientConfig::default();
            let (mut incoming_messages, client) =
                TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

            let join_handle = tokio::spawn(async move {
                while let Some(message) = incoming_messages.recv().await {
                    message_writer.try_send(message).unwrap();
                }
            });

            client
                .join(channel_name)
                .expect("TwitchMsgReader::new: Could not join Twitch channel");

            join_handle.await.unwrap();
        });

        Self {
            _rt,
            _background_task,
            message_receiver: message_reader,
        }
    }

    /// Returns some Notification if one was found from the Twitch
    /// chat, or None otherwise.
    pub fn read(&mut self) -> Option<Notification> {
        let msg_status = self.message_receiver.try_recv();

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

/// Converts Notifications from Twitch messages into Msgs to be
/// shown if found.
pub fn convert_notification_to_msg(
    mut notification_reader: EventReader<Notification>,
    mut msg_writer: EventWriter<ChatMsg>,
) {
    for notification in notification_reader.read() {
        let found_msg = notification.as_msg();
        if found_msg.is_none() {
            continue;
        }

        msg_writer.send(found_msg.unwrap());
    }
}
