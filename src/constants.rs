use std::sync::LazyLock;

pub const STREAMER_USERNAME: &str = "threevprime";
pub const STREAMER_NAME: &str = "Threev";
pub const USER_ID: &str = "453815125";
pub static OAUTH_TOKEN: LazyLock<String> =
    LazyLock::new(|| std::env::var("ACCESS_TOKEN").unwrap_or_default());
pub static CLIENT_ID: LazyLock<String> =
    LazyLock::new(|| std::env::var("CLIENT_ID").unwrap_or_default());
pub const EVENTSUB_WEBSOCKET_URL: &str = "wss://eventsub.wss.twitch.tv/ws";
pub static DATABASE_FILEPATH: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/twitch_database.db".to_string())
});
pub const BANNABLE_SENTENCES: [&str; 3] = [
    "streamboo .com",
    "streamboo . com",
    "I followed on twitter follow back",
];
