use std::sync::LazyLock;

pub static STREAMER_USERNAME: LazyLock<String> =
    LazyLock::new(|| std::env::var("STREAMER_USERNAME").unwrap_or_default());
pub static STREAMER_NAME: LazyLock<String> =
    LazyLock::new(|| std::env::var("STREAMER_NAME").unwrap_or_default());
pub static USER_ID: LazyLock<String> =
    LazyLock::new(|| std::env::var("USER_ID").unwrap_or_default());
pub static OAUTH_TOKEN: LazyLock<String> =
    LazyLock::new(|| std::env::var("ACCESS_TOKEN").unwrap_or_default());
pub static CLIENT_ID: LazyLock<String> =
    LazyLock::new(|| std::env::var("CLIENT_ID").unwrap_or_default());

pub const API_BASE_URL: &str = "https://api.twitch.tv/helix";
pub const EVENTSUB_WEBSOCKET_URL: &str = "wss://eventsub.wss.twitch.tv/ws";
pub static DATABASE_FILEPATH: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/twitch_database.db".to_string())
});
pub const BANNABLE_SENTENCES: [&str; 3] = [
    "streamboo .com",
    "streamboo . com",
    "I followed on twitter follow back",
];
