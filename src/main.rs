mod bot;
mod constants;
mod handle_user_data;
mod requests;
mod responses;
mod types;
mod utils;
mod websocket;

use std::sync::Arc;
use std::time::Duration;

use tokio::time::{interval, sleep};

use bot::TwitchBot;
use constants::{CLIENT_ID, OAUTH_TOKEN, USER_ID};
use websocket::start_web_socket_client;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Err(error) = handle_user_data::init_database() {
        eprintln!("Could not set up the database: {error}");
        std::process::exit(1);
    }

    let bot = Arc::new(TwitchBot::new(
        OAUTH_TOKEN.clone(),
        CLIENT_ID.clone(),
        USER_ID.clone(),
    ));

    bot.get_auth().await;

    // Twitch wants the token validated every hour
    let validator = Arc::clone(&bot);
    tokio::spawn(async move {
        let mut hourly = interval(Duration::from_secs(60 * 60));
        hourly.tick().await;
        loop {
            hourly.tick().await;
            validator.get_auth().await;
        }
    });

    loop {
        if let Err(error) = start_web_socket_client(Arc::clone(&bot)).await {
            eprintln!("WebSocket error: {error}");
        }
        eprintln!("Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}
