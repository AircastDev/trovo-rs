use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::collections::HashMap;

/// Holds a chat token obtained via the api to authenticate
/// a chat session.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatToken {
    /// Chat token to authenticate to chat with
    pub token: String,
}

/// Messages that can be sent over the socket to interact
/// with the Trovo chat api
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum ChatSocketMessage {
    /// Authenticate the chat session
    Auth {
        /// Sent back in responses, used to map a request to a reply
        nonce: String,

        /// Object containing the chat token to authenticate with
        data: ChatToken,
    },

    /// Sent by Trovo to acknowledge the auth message
    Response {
        /// Sent back in responses, used to map a request to a reply
        nonce: String,
    },

    /// A simple ping message to keep the chat socket alive
    Ping {
        /// Sent back in responses, used to map a request to a reply
        nonce: String,
    },

    /// Response to sending a ping message.
    Pong {
        /// Sent back in responses, used to map a request to a reply
        nonce: String,

        /// Ping response data
        data: PongMessageData,
    },

    /// Sent by trovo when a chat message is sent in chat.
    /// May contain mulptiple chat messages at a time.
    ///
    /// This is also sent on connection to a channel if there were recent chat messages.
    Chat {
        /// Contains information about which channel the messages were sent in.
        ///
        /// Seemingly not present on historic chat messages.
        channel_info: Option<ChannelInfo>,

        /// Chat message data
        data: ChatMessageData,
    },
}

/// Data sent back in response to a Ping message
#[derive(Debug, Serialize, Deserialize)]
pub struct PongMessageData {
    /// Interval in seconds that the server advises you to ping it.
    pub gap: u64,
}

/// Channel information sent with a chat message
#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// Id of the channel the chat messages were sent in
    pub channel_id: String,
}

/// List of chat messages that were sent
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageData {
    /// Message container ID. This is different from message ID. One message
    /// container may contain one or multiple messages.
    pub eid: String,

    /// A list of chats. One chat message may contain multiple chats.
    #[serde(default)]
    pub chats: Vec<ChatMessage>,
}

/// Type of the chat message
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum ChatMessageType {
    /// Normal chat messages.
    Normal = 0,

    /// Spells, including: mana spells, elixir spells
    Spell = 5,

    /// Magic chat - super cap chat
    MagicSuperCap = 6,

    /// Magic chat - colorful chat
    MagicColorful = 7,

    /// Magic chat - spell chat
    MagicSpell = 8,

    /// Magic chat - bullet screen chat
    MagicBulletScreen = 9,

    /// Subscription message. Shows when someone subscribes to the channel.
    Subscription = 5001,

    /// System message.
    System = 5002,

    /// Follow message. Shows when someone follows the channel.
    Follow = 5003,

    /// Welcome message when viewer joins the channel.
    Welcome = 5004,

    /// Gift sub message. When a user randomly sends gift subscriptions to one or more users in the channel.
    GiftSub = 5005,

    /// Gift sub message. The detailed messages when a user sends a gift subscription to another user.
    GiftSubDetailed = 5006,

    /// Activity / events message. For platform level events.
    Event = 5007,

    /// Welcome message when users join the channel from raid.
    Raid = 5008,

    /// Custom Spells
    CustomSpell = 5009,
}

/// A single chat message
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Type of chat message.
    #[serde(rename = "type")]
    pub type_: ChatMessageType,

    /// Content of the message. Now gift message add new fields, including gift_id,
    /// gift_value(unit price of a gift) and value_type(currency type, like Elixir, Mana and so on).
    pub content: String,

    /// Display name of the sender.
    pub nick_name: String,

    /// URL of the sender’s profile picture.
    pub avatar: Option<String>,

    /// The subscription level of the user in the channel. “sub_L1” for tier 1 subscriber.
    pub sub_lv: Option<String>,

    /// The list of badge names of the sender.
    #[serde(default)]
    pub medals: Vec<String>,

    /// The list of decoration names of sender.
    #[serde(default)]
    pub decos: Vec<String>,

    /// The list of roles of the message sender. One user can have multiple roles, for example: “roles”:[“mod”, “follower”]
    #[serde(default)]
    pub roles: Vec<String>,

    /// ID of the message.
    pub message_id: String,

    /// User ID of the sender.
    pub sender_id: i64,

    /// Extra info of chat
    #[serde(default)]
    pub content_data: HashMap<String, serde_json::Value>,

    /// The list of role of the message sender which is a json string. Different from "roles", "custom_role"
    /// contains more information. However, if you just need the role names, use "roles" instead.
    pub custom_role: Option<String>,
}

/// Payload for the send chat message request
#[derive(Debug, Serialize, Deserialize)]
pub struct SendChatMessagePayload {
    /// The message contents
    pub content: String,

    /// The channel to send the message in, if None then it will send to the user's own channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
}
