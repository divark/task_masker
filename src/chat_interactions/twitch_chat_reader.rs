use bevy::prelude::*;

use rand::seq::IteratorRandom;

use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{
    ServerMessage::Privmsg,
    ServerMessage::{self, UserNotice},
    UserNoticeEvent,
};
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

use crate::chat_interactions::plugins::CHANNEL_NAME;
use crate::entities::chatter::ChatMsg;
use crate::entities::crop::CropState;
use crate::entities::fruit::FruitState;
use crate::entities::subscriber::SubscriberMsg;
use crate::entities::TriggerQueue;

/// Represents a chatter's role sending
/// messages from Twitch.
pub enum TwitchRole {
    Chatter,
    Subscriber,
    Streamer,
}

/// Represents common Twitch events that
/// could take place in a stream.
pub enum TwitchEvent {
    Donation,
}

/// Represents the different type of
/// Twitch events for some Notification.
pub enum NotificationType {
    Msg(TwitchRole),
    Event(TwitchEvent),
}

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
    pub fn as_chat_msg(&self) -> Option<ChatMsg> {
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

    /// Converts the contents of the Notification into a
    /// SubscriberMsg if possible, or returns None otherwise.
    pub fn as_subscriber_msg(&self) -> Option<SubscriberMsg> {
        if let Privmsg(current_msg) = &self.msg {
            let speaker_name = current_msg.sender.name.clone();
            let speaker_msg = current_msg.message_text.clone();

            Some(SubscriberMsg {
                name: speaker_name,
                msg: speaker_msg,
            })
        } else {
            None
        }
    }

    /// Returns whether the message has a donation attached to it
    /// or not.
    pub fn has_bits(&self) -> bool {
        if let Privmsg(current_msg) = &self.msg {
            let has_bits = current_msg.bits.is_some();

            return has_bits;
        } else {
            false
        }
    }

    /// Returns whether the message has a subscribed event attached
    /// to it or not.
    pub fn has_subscribed(&self) -> bool {
        if let UserNotice(current_msg) = &self.msg {
            let has_subscribed = match &current_msg.event {
                UserNoticeEvent::SubOrResub { .. } => true,
                UserNoticeEvent::SubGift { .. } => true,
                _ => false,
            };

            return has_subscribed;
        } else {
            false
        }
    }

    /// Returns the type of chat message that was captured
    /// from Twitch.
    pub fn msg_type(&self) -> Option<NotificationType> {
        if let Privmsg(current_msg) = &self.msg {
            let speaker_name = current_msg.sender.name.clone();
            if speaker_name == CHANNEL_NAME {
                return Some(NotificationType::Msg(TwitchRole::Streamer));
            }

            let is_subscriber = current_msg.badge_info.len() > 0;
            if is_subscriber {
                return Some(NotificationType::Msg(TwitchRole::Subscriber));
            }

            Some(NotificationType::Msg(TwitchRole::Chatter))
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

/// Converts Notifications from Twitch messages into a Message to be
/// shown if found.
pub fn convert_notification_to_msg(
    mut notification_reader: EventReader<Notification>,
    mut chat_msg_writer: EventWriter<ChatMsg>,
    mut subscriber_msg_writer: EventWriter<SubscriberMsg>,
) {
    for notification in notification_reader.read() {
        let found_msg_type = notification.msg_type();
        if found_msg_type.is_none() {
            continue;
        }

        let msg_type = found_msg_type.unwrap();
        match msg_type {
            NotificationType::Msg(TwitchRole::Chatter) => {
                chat_msg_writer.send(notification.as_chat_msg().unwrap());
            }
            NotificationType::Msg(TwitchRole::Subscriber) => {
                subscriber_msg_writer.send(notification.as_subscriber_msg().unwrap());
            }
            _ => unimplemented!(),
            //NotificationType::Msg(TwitchRole::Streamer) => { streamer_msg_writer.send(notification.as_streamer_msg().unwrap()); },
        };

        let found_msg = notification.as_chat_msg();
        if found_msg.is_none() {
            continue;
        }

        chat_msg_writer.send(found_msg.unwrap());
    }
}

/// Converts Notifications from Twitch messages into an Event to be
/// triggered if found.
pub fn convert_notification_to_event(
    mut notification_reader: EventReader<Notification>,
    mut fruit_queues: Query<&mut TriggerQueue, (With<FruitState>, Without<CropState>)>,
    mut crop_queues: Query<&mut TriggerQueue, (With<CropState>, Without<FruitState>)>,
) {
    if fruit_queues.is_empty() || crop_queues.is_empty() {
        return;
    }

    for notification in notification_reader.read() {
        let mut random_fruit_queue = fruit_queues
            .iter_mut()
            .choose(&mut rand::thread_rng())
            .expect("convert_notification_to_event: Fruit should exist.");

        let mut random_crop_queue = crop_queues
            .iter_mut()
            .choose(&mut rand::thread_rng())
            .expect("Crop should exist by now.");

        if notification.has_bits() {
            random_fruit_queue.push_back(());
        }

        if notification.has_subscribed() {
            random_crop_queue.push_back(());
        }
    }
}
