![Crates.io](https://img.shields.io/crates/l/trovo)
[![Crates.io](https://img.shields.io/crates/v/trovo)](https://crates.io/crates/trovo)
[![Docs.rs](https://docs.rs/trovo/badge.svg)](https://docs.rs/trovo)
[![Workflow Status](https://github.com/AircastDev/trovo-rs/workflows/main/badge.svg)](https://github.com/AircastDev/trovo-rs/actions?query=workflow%3A%22main%22)

# trovo-rs

A Rust api client for [Trovo](https://trovo.live).

Currently supports connecting to chat, sending messages, looking up users via username, and fetching
channel information, more will be added as the crate develops.

### Example

Find a user by username and then connect to their chat.

```rust
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
```

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
