# Threev's chat bot (Rust)

Rust rewrite of the TypeScript Twitch chat bot. Uses EventSub over WebSocket and SQLite for user data.

## For anyone who wants to use this for themselves go to `src/constants.rs`

## Setup

1. Get your token for the bot.
    - You can use this site to get a token: https://twitchtokengenerator.com/
2. Copy `.env.example` to `.env` and fill in `ACCESS_TOKEN` and `CLIENT_ID`.
   The token must be a **user** token for the streamer account with these scopes:
   `user:read:chat user:write:chat channel:read:redemptions moderator:manage:shoutouts moderator:manage:banned_users`
3. Run it:
    - Locally: `cargo run`
    - Docker: `docker compose up -d --build`

The database is created automatically at `./data/twitch_database.db` (mounted as a volume in Docker).

## Bot Features

### Commands

- [x] !ship <user> - Shows the compatability between you and another user. Defaults to yourself

### Redeems

- [x] First / Second / Third - redeem for the first three viewers of the stream (with counters)
- [x] Daily - redeem for the daily viewer of the stream (with counter)

### Auto-Stuff

- [x] Automatically shout out incoming raiders
- [x] Automatically reply to people who have "I am, I'm, Im, i am, i'm, im" in their message