mod api;
mod bot;
mod constants;
mod handle_user_data;
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

    // Start Axum API
    let app = api::create_router(Arc::clone(&bot));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:42069")
        .await
        .expect("Could not bind API listener");

    println!("API listening on http://localhost:42069");
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("Axum server failed: {error}");
            std::process::exit(1);
        }
    });

    // Twitch token validation every hour
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
