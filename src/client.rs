use crate::{
    ApiError, ChannelInfo, ClientIdProvider, ErrorStatus, GetChannelByIdPayload, GetUsersPayload,
    GetUsersResponse, RequestError, User,
};
use std::time::Duration;

/// Entrypoint for making requests to the Trovo api.
#[derive(Debug, Clone)]
pub struct Client<A> {
    pub(crate) http: reqwest::Client,
    pub(crate) auth_provider: A,
}

impl<A> Client<A> {
    /// Creates a new default trovo client.
    /// If you are already using reqwest in your program, it is advisable
    /// to use [`Client::from_reqwest`] instead to allow for connection
    /// pool sharing.
    ///
    /// # Panics
    ///
    /// This method panics if a TLS backend cannot be initialized, or the resolver cannot load the system configuration.
    pub fn new(auth_provider: A) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            auth_provider,
        }
    }

    /// Creates a new trovo client using the provided reqwest client.
    ///
    /// This can be useful to allow sharing of a reqwest client's connection pool
    /// across your program
    pub fn from_reqwest(http: reqwest::Client, auth_provider: A) -> Self {
        Self {
            http,
            auth_provider,
        }
    }
}

impl<A> Client<A>
where
    A: ClientIdProvider,
{
    /// Gets a list of user’s channel id, user id, and nickname, by usernames.
    ///
    /// Note: Even if just one of the usernames doesn't exist, the result will be
    /// an empty vec due to api limitations.
    pub async fn users(&self, usernames: Vec<String>) -> Result<Vec<User>, RequestError> {
        let res = self
            .http
            .post("https://open-api.trovo.live/openplatform/getusers")
            .header("Client-ID", self.auth_provider.client_id())
            .json(&GetUsersPayload { user: usernames })
            .send()
            .await?;

        if ApiError::can_handle_code(res.status()) {
            let err: ApiError = res.json().await.unwrap_or_default();

            if err.status == ErrorStatus::InvalidParameters {
                return Ok(vec![]);
            } else {
                return Err(RequestError::ApiError(err));
            }
        }

        let response: GetUsersResponse = res.error_for_status()?.json().await?;
        Ok(response.users)
    }

    /// Gets a user’s channel id, user id, and nickname, by username.
    ///
    /// Returns None if the user was not found
    pub async fn user(&self, username: impl Into<String>) -> Result<Option<User>, RequestError> {
        let mut users = self.users(vec![username.into()]).await?;

        if !users.is_empty() {
            Ok(Some(users.remove(0)))
        } else {
            Ok(None)
        }
    }

    /// Gets channel information for the given id
    ///
    /// Returns None if the channel was not found
    pub async fn channel_by_id(
        &self,
        channel_id: impl Into<String>,
    ) -> Result<Option<ChannelInfo>, RequestError> {
        let res = self
            .http
            .post("https://open-api.trovo.live/openplatform/channels/id")
            .header("Client-ID", self.auth_provider.client_id())
            .json(&GetChannelByIdPayload {
                channel_id: channel_id.into(),
            })
            .send()
            .await?;

        if ApiError::can_handle_code(res.status()) {
            let err: ApiError = res.json().await.unwrap_or_default();
            return Err(RequestError::ApiError(err));
        }

        let channel: ChannelInfo = res.error_for_status()?.json().await?;
        Ok(if channel.username.is_empty() {
            // Trovo api returns a nulled out channel if it can't be found, username is probably
            // never legitimately blank
            None
        } else {
            Some(channel)
        })
    }
}
