use crate::{
    access_token,
    auth::{AccessTokenProvider, ClientIdProvider},
    chat::{ChatConnectError, ChatMessageStream, ChatToken},
    Client,
};
use reqwest::header;
use std::{error::Error, fmt::Display};

impl<A> Client<A>
where
    A: ClientIdProvider,
{
    /// Get a chat token for the provided channel id
    pub async fn chat_token_for_channel(
        &self,
        channel_id: impl AsRef<str>,
    ) -> Result<ChatToken, reqwest::Error> {
        self.http
            .get(&format!(
                "https://open-api.trovo.live/openplatform/chat/channel-token/{}",
                channel_id.as_ref()
            ))
            .header("Client-ID", self.auth_provider.client_id())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    /// Connect to the given channel id and receive a stream of messages.
    pub async fn chat_messages_for_channel(
        &self,
        channel_id: impl AsRef<str>,
    ) -> Result<ChatMessageStream, ChatConnectError> {
        let token = self.chat_token_for_channel(channel_id).await?;
        ChatMessageStream::connect(token).await
    }
}

impl<A> Client<A>
where
    A: AccessTokenProvider,
{
    /// Get a chat token for the authenticated user's channel
    pub async fn chat_token_for_user(&self) -> Result<ChatToken, ChatTokenForUserError<A::Error>> {
        let result = self
            .http
            .get("https://open-api.trovo.live/openplatform/chat/token")
            .header("Client-ID", self.auth_provider.client_id())
            .header(
                header::AUTHORIZATION,
                format!(
                    "OAuth {}",
                    access_token!(self.auth_provider, ChatTokenForUserError)
                ),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(result)
    }

    /// Connect to the authenticated user's channel and receive a stream of messages.
    pub async fn chat_messages_for_user(
        &self,
    ) -> Result<ChatMessageStream, ChatMessagesForUserError<A::Error>> {
        let token = self
            .chat_token_for_user()
            .await
            .map_err(ChatMessagesForUserError::ChatToken)?;
        let messages = ChatMessageStream::connect(token).await?;
        Ok(messages)
    }
}

/// Error that can happen on calls to [`Client::chat_token_for_user`]
#[derive(Debug)]
pub enum ChatTokenForUserError<E> {
    /// Error refreshing token
    RefreshToken(E),

    /// Error during request to server
    Http(reqwest::Error),
}

impl<E> From<reqwest::Error> for ChatTokenForUserError<E> {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl<E> Display for ChatTokenForUserError<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatTokenForUserError::RefreshToken(e) => e.fmt(f),
            ChatTokenForUserError::Http(e) => e.fmt(f),
        }
    }
}

impl<E> Error for ChatTokenForUserError<E>
where
    E: 'static + Error + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChatTokenForUserError::RefreshToken(e) => Some(e),
            ChatTokenForUserError::Http(e) => Some(e),
        }
    }
}

/// Error that can happen on calls to [`Client::chat_messages_for_user`]
#[derive(Debug)]
pub enum ChatMessagesForUserError<E> {
    /// Error fetching chat token
    ChatToken(ChatTokenForUserError<E>),

    /// Error during request to server
    ChatConnect(ChatConnectError),
}

impl<E> From<ChatConnectError> for ChatMessagesForUserError<E> {
    fn from(error: ChatConnectError) -> Self {
        Self::ChatConnect(error)
    }
}

impl<E> Display for ChatMessagesForUserError<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatMessagesForUserError::ChatToken(e) => e.fmt(f),
            ChatMessagesForUserError::ChatConnect(e) => e.fmt(f),
        }
    }
}

impl<E> Error for ChatMessagesForUserError<E>
where
    E: 'static + Error + Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChatMessagesForUserError::ChatToken(e) => Some(e),
            ChatMessagesForUserError::ChatConnect(e) => Some(e),
        }
    }
}
