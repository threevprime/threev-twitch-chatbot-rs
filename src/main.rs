mod api;
mod bot;
mod constants;
mod eventbus;
mod handle_user_data;
mod logger;
mod responses;
mod types;
mod utils;
mod websocket;

use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use bot::TwitchBot;
use constants::{CLIENT_ID, OAUTH_TOKEN, USER_ID};
use handle_user_data::get_commands;
use logger::init_logger;
use websocket::start_web_socket_client;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Err(error) = handle_user_data::init_database() {
        log::error!("Could not set up the database: {error}");
        std::process::exit(1);
    }
    init_logger();

    let bot = TwitchBot::new(OAUTH_TOKEN.clone(), CLIENT_ID.clone(), USER_ID.clone());
    match get_commands() {
        Ok(commands) => {
            for command in commands {
                log::debug!("Loading existing command {:?}", command.name());
                bot.add_command(command);
            }
        }
        Err(e) => log::error!("{:?}", e),
    }

    // Start Axum API
    let app = api::create_router(Arc::clone(&bot));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:42069")
        .await
        .expect("Could not bind API listener");

    log::info!("API listening on http://localhost:42069");
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            log::error!("Axum server failed: {error}");
            std::process::exit(1);
        }
    });

    loop {
        if let Err(error) = start_web_socket_client(Arc::clone(&bot)).await {
            log::error!("WebSocket error: {error}");
        }
        log::info!("Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}
