# Changelog

## Unreleased

### Updated

- reqwest updated to 0.12

## v0.5.0 (2022-07-25)

### Added

- Add `update_channel` function to client
- Add `send_time` field to chat messages

### Updated

- Now using edition 2021
- Updated various internal dependencies

## v0.4.0 (2021-07-26)

### Fixed

- `sender_id` may be None

## v0.3.0 (2021-07-26)

### Fixed

- Avatar can be null/missing on chat messages
- Handle gifp/webp fields being missing on some emotes

## v0.2.2 (2021-07-05)

### Added

- Add `emotes` function to client to fetch trovo emotes

## v0.2.1 (2021-07-05)

### Added

- Add `channel_by_id` function to client

### Fixed

- Deserialising of `chats` when various fields were null

## v0.2.0 (2021-07-05)

### Added

- Add `send_chat_message` function to client
- Added `AccessTokenOnly` struct that implements `AccessTokenProvider` for quick and dirty calls
  with an access token

### Changed

- `chat_token_for_channel` and `chat_token_for_user` now return `RequestError` and
  `AuthenticatedRequestError` respectively which give more information as to what the api error
  actually was.

### Fixed

- Deserialising of `chats` when connecting to a channel with no message history

## v0.1.0 (2021-07-05)

### Added

- Chat socket connection
- Methods to get user info by username
