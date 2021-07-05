use crate::{
    access_token,
    auth::{AccessTokenProvider, ClientIdProvider},
    chat::{ChatConnectError, ChatMessageStream, ChatToken, SendChatMessagePayload},
    ApiError, AuthenticatedRequestError, Client, RequestError,
};
use reqwest::header;
use std::{
    error::Error,
    fmt::{Debug, Display},
};
use thiserror::Error;

impl<A> Client<A>
where
    A: ClientIdProvider,
{
    /// Get a chat token for the provided channel id
    pub async fn chat_token_for_channel(
        &self,
        channel_id: impl AsRef<str>,
    ) -> Result<ChatToken, RequestError> {
        let res = self
            .http
            .get(&format!(
                "https://open-api.trovo.live/openplatform/chat/channel-token/{}",
                channel_id.as_ref()
            ))
            .header("Client-ID", self.auth_provider.client_id())
            .send()
            .await?;

        if ApiError::can_handle_code(res.status()) {
            let err: ApiError = res.json().await.unwrap_or_default();
            Err(RequestError::ApiError(err))
        } else {
            let response = res.error_for_status()?.json().await?;
            Ok(response)
        }
    }

    /// Connect to the given channel id and receive a stream of messages.
    pub async fn chat_messages_for_channel(
        &self,
        channel_id: impl AsRef<str>,
    ) -> Result<ChatMessageStream, ChatMessagesForChannelError> {
        let token = self.chat_token_for_channel(channel_id).await?;
        let messages = ChatMessageStream::connect(token).await?;
        Ok(messages)
    }
}

impl<A> Client<A>
where
    A: AccessTokenProvider,
{
    /// Get a chat token for the authenticated user's channel
    pub async fn chat_token_for_user(
        &self,
    ) -> Result<ChatToken, AuthenticatedRequestError<A::Error>> {
        let res = self
            .http
            .get("https://open-api.trovo.live/openplatform/chat/token")
            .header("Client-ID", self.auth_provider.client_id())
            .header(
                header::AUTHORIZATION,
                format!(
                    "OAuth {}",
                    access_token!(self.auth_provider, AuthenticatedRequestError)
                ),
            )
            .send()
            .await?;

        if ApiError::can_handle_code(res.status()) {
            let err: ApiError = res.json().await.unwrap_or_default();
            Err(AuthenticatedRequestError::ApiError(err))
        } else {
            let response = res.error_for_status()?.json().await?;
            Ok(response)
        }
    }

    /// Connect to the authenticated user's channel and receive a stream of messages.
    pub async fn chat_messages_for_user(
        &self,
    ) -> Result<ChatMessageStream, ChatMessagesForUserError<A::Error>> {
        let token = self
            .chat_token_for_user()
            .await
            .map_err(ChatMessagesForUserError::Request)?;
        let messages = ChatMessageStream::connect(token).await?;
        Ok(messages)
    }

    /// Send a chat message to a channel
    ///
    /// # Scopes
    ///
    /// ## Sending to your own channel
    ///
    /// Requires `chat_send_self`
    ///
    /// ## Sending to other channels
    ///
    /// To send a message as sender (user A) to channel (owned by user B), the application needs to
    /// get scopes `chat_send_self` of user A, and `send_to_my_channel` of user B.
    pub async fn send_chat_message(
        &self,
        channel_id: Option<String>,
        message: impl Into<String>,
    ) -> Result<(), AuthenticatedRequestError<A::Error>> {
        let res = self
            .http
            .post("https://open-api.trovo.live/openplatform/chat/send")
            .header("Client-ID", self.auth_provider.client_id())
            .header(
                header::AUTHORIZATION,
                format!(
                    "OAuth {}",
                    access_token!(self.auth_provider, AuthenticatedRequestError)
                ),
            )
            .json(&SendChatMessagePayload {
                content: message.into(),
                channel_id,
            })
            .send()
            .await?;

        if ApiError::can_handle_code(res.status()) {
            let err: ApiError = res.json().await.unwrap_or_default();
            Err(AuthenticatedRequestError::ApiError(err))
        } else {
            res.error_for_status()?;
            Ok(())
        }
    }
}

/// Error that can happen on calls to [`Client::chat_messages_for_user`]
#[derive(Debug)]
pub enum ChatMessagesForUserError<E>
where
    E: Display + Debug,
{
    /// Error fetching chat token
    Request(AuthenticatedRequestError<E>),

    /// Error during request to server
    ChatConnect(ChatConnectError),
}

impl<E> From<ChatConnectError> for ChatMessagesForUserError<E>
where
    E: Display + Debug,
{
    fn from(error: ChatConnectError) -> Self {
        Self::ChatConnect(error)
    }
}

impl<E> Display for ChatMessagesForUserError<E>
where
    E: Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatMessagesForUserError::Request(e) => Display::fmt(e, f),
            ChatMessagesForUserError::ChatConnect(e) => Display::fmt(e, f),
        }
    }
}

impl<E> Error for ChatMessagesForUserError<E>
where
    E: 'static + Error + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChatMessagesForUserError::Request(e) => Some(e),
            ChatMessagesForUserError::ChatConnect(e) => Some(e),
        }
    }
}

/// Error that can happen on calls to [`Client::chat_messages_for_user`]
#[derive(Debug, Error)]
pub enum ChatMessagesForChannelError {
    /// Error fetching chat token
    #[error(transparent)]
    Request(#[from] RequestError),

    /// Error during request to server
    #[error(transparent)]
    ChatConnect(#[from] ChatConnectError),
}
