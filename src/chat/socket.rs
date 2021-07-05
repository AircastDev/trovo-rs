use crate::chat::{
    ChatConnectError, ChatMessage, ChatMessageStreamError, ChatSocketMessage, ChatToken,
};
use async_tungstenite::{
    tokio::connect_async,
    tungstenite::{self, Message},
};
use futures::prelude::*;
use std::time::Duration;
use tokio::{
    select,
    sync::{mpsc, oneshot},
    time::sleep,
};
use tokio_util::sync::CancellationToken;

const CHAT_MESSAGES_BUFFER: usize = 32;
const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(30);

/// A stream of chat messages
#[derive(Debug)]
pub struct ChatMessageStream {
    cancellation_token: CancellationToken,
    messages: mpsc::Receiver<Result<ChatMessage, ChatMessageStreamError>>,
}

impl ChatMessageStream {
    /// Connect to trovo chat using the given chat token.
    ///
    /// See [`Client::chat_messages_for_channel`] and [`Client::chat_messages_for_user`]
    /// for fetching the token and connecting in one.
    pub async fn connect(chat_token: ChatToken) -> Result<ChatMessageStream, ChatConnectError> {
        let cancellation_token = CancellationToken::new();
        let (ws_stream, _) = connect_async("wss://open-chat.trovo.live/chat").await?;
        let (mut writer, reader) = ws_stream.split();
        let (socket_messages_sender, socket_messages_receiver) = mpsc::channel(1);
        let (chat_messages_sender, chat_messages_receiver) = mpsc::channel(CHAT_MESSAGES_BUFFER);
        let (auth_response_sender, auth_response_receiver) = oneshot::channel();

        let auth_nonce = "authenticate".to_string(); // TODO randomly generate?

        let reader = SocketMessagesReader {
            reader,
            cancellation_token: cancellation_token.clone(),
            auth: (auth_nonce.clone(), Some(auth_response_sender)),
            chat_messages_sender: chat_messages_sender.clone(),
            socket_messages_sender,
            ping: Default::default(),
        };
        reader.spawn();

        let msg = serde_json::to_string(&ChatSocketMessage::Auth {
            nonce: auth_nonce,
            data: chat_token,
        })?;
        writer.send(msg.into()).await?;

        auth_response_receiver
            .await
            .map_err(|_| ChatConnectError::SocketClosed)??;

        let writer = SocketMessagesWriter {
            writer,
            cancellation_token: cancellation_token.clone(),
            socket_messages_receiver,
            chat_messages_sender,
        };
        writer.spawn();

        Ok(ChatMessageStream {
            cancellation_token,
            messages: chat_messages_receiver,
        })
    }

    /// Close the chat socket, causing any further calls to `next()` to return `None`.
    ///
    /// Automatically called on drop. Calling multiple times has no effect.
    pub fn close(&self) {
        self.cancellation_token.cancel()
    }
}

impl Stream for ChatMessageStream {
    type Item = Result<ChatMessage, ChatMessageStreamError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.messages.poll_recv(cx)
    }
}

impl Drop for ChatMessageStream {
    fn drop(&mut self) {
        self.close()
    }
}

#[derive(Debug, PartialEq)]
enum Continuation {
    Continue,
    Stop,
}

#[derive(Debug)]
struct Ping {
    interval: Duration,
    iteration: u64,

    /// The last iteration that we got a Pong response to
    acknowledged: u64,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            interval: DEFAULT_PING_INTERVAL,
            iteration: 0,
            acknowledged: 0,
        }
    }
}

struct SocketMessagesReader<R> {
    cancellation_token: CancellationToken,
    reader: R,
    chat_messages_sender: mpsc::Sender<Result<ChatMessage, ChatMessageStreamError>>,
    socket_messages_sender: mpsc::Sender<ChatSocketMessage>,
    auth: (
        String,
        Option<oneshot::Sender<Result<(), ChatConnectError>>>,
    ),
    ping: Ping,
}

impl<R> SocketMessagesReader<R>
where
    R: 'static + Stream<Item = Result<Message, tungstenite::Error>> + Send + Unpin,
{
    fn spawn(mut self) {
        tokio::spawn(async move {
            loop {
                match self.next().await {
                    Ok(Continuation::Stop) => {
                        trace!("socket reader exited gracefully");
                        break;
                    }
                    Err(err) => {
                        error!(?err, "socket reader errored");
                        self.chat_messages_sender.send(Err(err)).await.ok();
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    async fn next(&mut self) -> Result<Continuation, ChatMessageStreamError> {
        select! {
            _ = self.cancellation_token.cancelled() => {
                Ok(Continuation::Stop)
            }
            _ = sleep(self.ping.interval) => {
                self.ping.iteration += 1;

                // Are we missing 2 pongs?
                if (self.ping.iteration - self.ping.acknowledged) > 2 {
                    return Err(ChatMessageStreamError::PingTimeout);
                }

                let msg = ChatSocketMessage::Ping { nonce: self.ping.iteration.to_string() };
                trace!(?msg, "sending ping");
                match self.socket_messages_sender.send(msg).await {
                    Ok(_) => Ok(Continuation::Continue),
                    Err(_) => Ok(Continuation::Stop),
                }
            }
            Some(msg) = self.reader.next() => {
                self.handle_message(msg?).await
            }
            else => {
                Ok(Continuation::Stop)
            }
        }
    }

    async fn handle_message(
        &mut self,
        msg: Message,
    ) -> Result<Continuation, ChatMessageStreamError> {
        trace!(?msg, "incoming websocket message");
        match msg {
            Message::Text(text) => {
                let msg = serde_json::from_str(&text)?;
                Ok(self.handle_socket_message(msg).await)
            }
            Message::Binary(bytes) => {
                let msg = serde_json::from_slice(bytes.as_slice())?;
                Ok(self.handle_socket_message(msg).await)
            }
            Message::Ping(_) => todo!(),
            Message::Pong(_) => todo!(),
            Message::Close(reason) => Err(ChatMessageStreamError::SocketClosed(reason)),
        }
    }

    async fn handle_socket_message(&mut self, msg: ChatSocketMessage) -> Continuation {
        debug!(?msg, "incoming chat socket message");
        match msg {
            ChatSocketMessage::Response { nonce } => {
                if self.auth.0 == nonce {
                    if let Some(auth) = self.auth.1.take() {
                        auth.send(Ok(())).ok();
                    }
                }
                Continuation::Continue
            }
            ChatSocketMessage::Pong { nonce, data } => {
                let iteration: u64 = match nonce.parse() {
                    Ok(v) => v,
                    Err(err) => {
                        warn!(?err, "failed to parse pong nonce as u64, ignoring...");
                        return Continuation::Continue;
                    }
                };
                debug!(?iteration, "received pong");
                // Ignore potentially delayed responses from any old pings
                if iteration > self.ping.acknowledged {
                    self.ping.acknowledged = iteration;
                    self.ping.interval = Duration::from_secs(data.gap);
                }
                Continuation::Continue
            }
            ChatSocketMessage::Chat {
                channel_info: _,
                data,
            } => {
                for chat in data.chats {
                    if self.chat_messages_sender.send(Ok(chat)).await.is_err() {
                        // Messages receiver must have been dropped and so we just need to cleanup
                        return Continuation::Stop;
                    }
                }
                Continuation::Continue
            }
            _ => unreachable!(),
        }
    }
}

impl<R> Drop for SocketMessagesReader<R> {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

struct SocketMessagesWriter<W> {
    cancellation_token: CancellationToken,
    writer: W,
    socket_messages_receiver: mpsc::Receiver<ChatSocketMessage>,
    chat_messages_sender: mpsc::Sender<Result<ChatMessage, ChatMessageStreamError>>,
}

impl<W> SocketMessagesWriter<W>
where
    W: 'static + Sink<Message, Error = tungstenite::Error> + Send + Unpin,
{
    fn spawn(mut self) {
        tokio::spawn(async move {
            loop {
                match self.next().await {
                    Ok(Continuation::Stop) => {
                        trace!("socket writer exited gracefully");
                        break;
                    }
                    Err(err) => {
                        error!(?err, "socket writer errored");
                        self.chat_messages_sender.send(Err(err)).await.ok();
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    async fn next(&mut self) -> Result<Continuation, ChatMessageStreamError> {
        select! {
            _ = self.cancellation_token.cancelled() => {
                Ok(Continuation::Stop)
            }
            Some(msg) = self.socket_messages_receiver.recv() => {
                self.handle_message(msg).await?;
                Ok(Continuation::Continue)
            }
            else => {
                Ok(Continuation::Stop)
            }
        }
    }

    async fn handle_message(
        &mut self,
        msg: ChatSocketMessage,
    ) -> Result<(), ChatMessageStreamError> {
        trace!(?msg, "outgoing websocket message");
        let msg = serde_json::to_string(&msg)?;
        self.writer.send(msg.into()).await?;
        Ok(())
    }
}

impl<W> Drop for SocketMessagesWriter<W> {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::PongMessageData;

    #[tokio::test]
    async fn ping_pong() {
        let cancellation_token = CancellationToken::new();
        let (socket_messages_sender, _) = mpsc::channel(1);
        let (chat_messages_sender, _) = mpsc::channel(CHAT_MESSAGES_BUFFER);
        let (mut fake_sender, fake_receiver) =
            futures::channel::mpsc::channel::<Result<Message, tungstenite::Error>>(1);
        let mut reader = SocketMessagesReader {
            cancellation_token,
            reader: fake_receiver,
            chat_messages_sender,
            socket_messages_sender,
            auth: ("authenticate".to_string(), None),
            ping: Ping {
                interval: DEFAULT_PING_INTERVAL,
                iteration: 1,
                acknowledged: 0,
            },
        };

        // Should acknowledge pongs
        let msg = serde_json::to_string(&ChatSocketMessage::Pong {
            nonce: 1.to_string(),
            data: PongMessageData { gap: 10 },
        })
        .unwrap();
        fake_sender.send(Ok(msg.into())).await.unwrap();
        assert_eq!(reader.ping.acknowledged, 0);
        assert_eq!(reader.ping.interval, DEFAULT_PING_INTERVAL);
        assert!(matches!(reader.next().await, Ok(Continuation::Continue)));
        assert_eq!(reader.ping.acknowledged, 1);
        assert_eq!(reader.ping.interval, Duration::from_secs(10));

        // Invalid nonce shouldn't kill the reader
        let msg = serde_json::to_string(&ChatSocketMessage::Pong {
            nonce: (-2).to_string(),
            data: PongMessageData { gap: 20 },
        })
        .unwrap();
        reader.ping.interval = DEFAULT_PING_INTERVAL;
        fake_sender.send(Ok(msg.into())).await.unwrap();
        assert!(matches!(reader.next().await, Ok(Continuation::Continue)));
        assert_eq!(reader.ping.acknowledged, 1);
        assert_eq!(reader.ping.interval, DEFAULT_PING_INTERVAL);

        // Should ignore backwards nonces
        let msg = serde_json::to_string(&ChatSocketMessage::Pong {
            nonce: 2.to_string(),
            data: PongMessageData { gap: 20 },
        })
        .unwrap();
        fake_sender.send(Ok(msg.into())).await.unwrap();
        reader.ping.interval = DEFAULT_PING_INTERVAL;
        reader.ping.acknowledged = 5;
        reader.ping.iteration = 6;
        assert!(matches!(reader.next().await, Ok(Continuation::Continue)));
        assert_eq!(reader.ping.acknowledged, 5);
        assert_eq!(reader.ping.interval, DEFAULT_PING_INTERVAL);
    }

    #[test]
    fn cancel_on_drop() {
        let cancellation_token = CancellationToken::new();
        let (_, messages) = mpsc::channel(CHAT_MESSAGES_BUFFER);

        assert!(!cancellation_token.is_cancelled());
        drop(ChatMessageStream {
            cancellation_token: cancellation_token.clone(),
            messages,
        });
        assert!(cancellation_token.is_cancelled());
    }
}
