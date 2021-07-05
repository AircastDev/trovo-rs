use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User details returned by [`Client::users`](crate::Client::users)
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// Unique id of a user.
    pub user_id: String,

    /// Unique id of a channel.
    pub channel_id: String,

    /// The username of a user. Unique across Trovo platform. Username is the last part of the streamer’s channel url.
    pub username: String,

    /// The display name of a user, displayed in chats, channels and all across Trovo. This could be different from username.
    pub nickname: String,
}

/// Payload for the get users api
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUsersPayload {
    /// A list of valid usernames that you want to request for. Not case sensitive.
    pub user: Vec<String>,
}

/// Response for the get users api
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUsersResponse {
    /// The list of user info for each username requested.
    pub users: Vec<User>,
}

/// Payload for the get channel info by id api
#[derive(Debug, Serialize, Deserialize)]
pub struct GetChannelByIdPayload {
    /// Channel id indicating which channel you are requesting.
    pub channel_id: String,
}

/// Response for the get users api
#[derive(Debug, Deserialize)]
pub struct ChannelInfo {
    /// If the channel is currently live streaming.
    pub is_live: bool,

    /// The id of the game category.
    pub category_id: String,

    /// Text name of the category.
    pub category_name: String,

    /// Current title of the channel.
    pub live_title: String,

    /// Audience type.
    pub audi_type: AudienceType,

    /// Language of the channel in in ISO 2 (2 letter language code)
    pub language_code: String,

    /// URL of the thumbnail. Empty thumbnail means the thumbnail from the previous stream has
    /// expired.
    pub thumbnail: String,

    /// Number of current viewers
    pub current_viewers: u64,

    /// Number of followers
    pub followers: u64,

    /// Profile information of the streamer
    pub streamer_info: String,

    /// Url of the streamer’s profile picture
    pub profile_pic: String,

    /// URL of the channel
    pub channel_url: String,

    /// Timestamp of the streamer creation time
    #[serde(with = "serde_with::chrono::datetime_utc_ts_seconds_from_any")]
    pub created_at: DateTime<Utc>,

    /// Count of subscribers
    pub subscriber_num: u64,

    /// Username of the channel’s streamer. Also the last part of the channel url.
    pub username: String,

    /// Social media links of the streamer.
    pub social_links: Vec<SocialLink>,

    /// The latest streaming start time of a given channel.
    #[serde(with = "serde_with::chrono::datetime_utc_ts_seconds_from_any")]
    pub started_at: DateTime<Utc>,

    /// The latest streaming end time of a given channel.
    #[serde(with = "serde_with::chrono::datetime_utc_ts_seconds_from_any")]
    pub ended_at: DateTime<Utc>,
}

/// Audience type of a channel
#[derive(Debug, Serialize, Deserialize)]
pub enum AudienceType {
    /// Family friendly
    #[serde(rename = "CHANNEL_AUDIENCE_TYPE_FAMILYFRIENDLY")]
    FamilyFriendly,

    /// Teen
    #[serde(rename = "CHANNEL_AUDIENCE_TYPE_TEEN")]
    Teen,

    /// 18+
    #[serde(rename = "CHANNEL_AUDIENCE_TYPE_EIGHTEENPLUS")]
    EighteenPlus,
}

/// Social media link for a channel
#[derive(Debug, Serialize, Deserialize)]
pub struct SocialLink {
    /// Social media platform
    #[serde(rename = "type")]
    type_: String,

    /// Url to the account on the given platform
    url: String,
}
