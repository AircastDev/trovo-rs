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
