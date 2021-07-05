# Changelog

## v0.2.0 (2021-07-05)

### Added

-   Add `send_chat_message` function to client
-   Added `AccessTokenOnly` struct that implements `AccessTokenProvider` for quick and dirty calls
    with an access token

### Changed

-   `chat_token_for_channel` and `chat_token_for_user` now return `RequestError` and
    `AuthenticatedRequestError` respectively which give more information as to what the api error
    actually was.

### Fixed

-   Deserialising of `chats` when connecting to a channel with no message history

## v0.1.0 (2021-07-05)

### Added

-   Chat socket connection
-   Methods to get user info by username
