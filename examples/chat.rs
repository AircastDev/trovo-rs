use futures::prelude::*;
use std::{env, error::Error};
use trovo::ClientId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client_id = env::var("CLIENT_ID").expect("missing CLIENT_ID env var");
    let username = env::var("USER_NAME").expect("missing USER_NAME env var");

    let client = trovo::Client::new(ClientId::new(client_id));

    println!("looking up user '{}'", username);
    let user = client
        .user(username)
        .await?
        .expect("no user found for the given username");
    println!("found user {:#?}", user);

    let mut messages = client.chat_messages_for_channel(&user.channel_id).await?;
    println!("listening for chat messages");
    while let Some(msg) = messages.next().await {
        let msg = msg?;
        println!("[{}] {}", msg.nick_name, msg.content);
    }

    Ok(())
}
