# Threev's chat bot (Rust)

Rust rewrite of the TypeScript Twitch chat bot. Uses EventSub over WebSocket and SQLite for user data.

## For anyone who wants to use this for themselves go to `src/constants.rs`

## Setup

1. Copy `.env.example` to `.env` and fill in `ACCESS_TOKEN` and `CLIENT_ID`.
   The token must be a **user** token for the streamer account with these scopes:
   `user:read:chat user:write:chat channel:read:redemptions moderator:manage:shoutouts moderator:manage:banned_users`
2. Run it:
    - Locally: `cargo run`
    - Docker: `docker compose up -d --build`

The database is created automatically at `./data/twitch_database.db` (mounted as a volume in Docker).

## Bot Features

### Commands

- [x] !help - Lists all commands
- [x] !github - Gives github URL
- [x] !raid - Gives the non sub raid message
- [x] !sussy - Shows how sussy a person is
- [x] !rickrolled - yes
- [x] !ship [username_1] [username_2] - Shows the compatability of 2 users

### Redeems

- [x] First / Second / Third - redeem for the first three viewers of the stream (with counters)
- [x] Daily - redeem for the daily viewer of the stream (with counter)

### Auto-Stuff

- [x] Automatically shout out incoming raiders
- [x] Automatically reply to people who have "I am, I'm, Im, i am, i'm, im" in their message
- [x] Automatically ban people that want to do the free viewers message
- [x] Automatically ban people that say "I followed on twitter follow back"
