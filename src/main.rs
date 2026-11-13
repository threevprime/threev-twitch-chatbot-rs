mod bot;
mod constants;
mod event_sub_listeners;
mod handle_user_data;
mod requests;
mod responses;
mod types;
mod utils;

use std::error::Error;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::Value;
use tokio::time::{interval, sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use constants::{EVENTSUB_WEBSOCKET_URL, USER_ID};
use event_sub_listeners::handle_registering_event_sub_listeners;
use requests::get_auth;
use responses::automatic::{handle_auto_bans, handle_dad_jokes, handle_raids};
use responses::commands::{shipping_users, simple_response_commands};
use responses::redeems::handle_redeems;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Err(error) = handle_user_data::init_database() {
        eprintln!("Could not set up the database: {error}");
        std::process::exit(1);
    }

    get_auth().await;

    // Twitch wants the token validated every hour
    tokio::spawn(async {
        let mut hourly = interval(Duration::from_secs(60 * 60));
        hourly.tick().await;
        loop {
            hourly.tick().await;
            get_auth().await;
        }
    });

    loop {
        if let Err(error) = start_web_socket_client().await {
            eprintln!("WebSocket error: {error}");
        }
        eprintln!("Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn start_web_socket_client() -> Result<(), Box<dyn Error>> {
    let (mut web_socket_client, _) = connect_async(EVENTSUB_WEBSOCKET_URL).await?;
    println!("WebSocket connection opened to {EVENTSUB_WEBSOCKET_URL}");

    // Only register listeners on a fresh session. After a session_reconnect
    // Twitch moves the existing subscriptions over to the new connection.
    let mut is_reconnect = false;
    let mut keepalive = Duration::from_secs(30);

    loop {
        let message = match timeout(keepalive, web_socket_client.next()).await {
            Err(_) => return Err("no message from Twitch within the keepalive window".into()),
            Ok(None) => return Err("connection closed".into()),
            Ok(Some(message)) => message?,
        };

        let text = match message {
            Message::Text(text) => text,
            Message::Close(frame) => return Err(format!("closed by Twitch: {frame:?}").into()),
            _ => continue,
        };

        let data: Value = serde_json::from_str(text.as_str())?;
        let session = &data["payload"]["session"];

        match data["metadata"]["message_type"]
            .as_str()
            .unwrap_or_default()
        {
            "session_welcome" => {
                let seconds = session["keepalive_timeout_seconds"].as_u64().unwrap_or(10);
                keepalive = Duration::from_secs(seconds + 5);

                if !is_reconnect {
                    let websocket_session_id = session["id"].as_str().unwrap_or_default();
                    handle_registering_event_sub_listeners(websocket_session_id).await;
                }
            }
            "session_reconnect" => {
                let reconnect_url = session["reconnect_url"]
                    .as_str()
                    .ok_or("session_reconnect without a reconnect_url")?;
                println!("Twitch asked us to reconnect, moving to the new connection");
                (web_socket_client, _) = connect_async(reconnect_url).await?;
                is_reconnect = true;
            }
            "notification" => {
                tokio::spawn(handle_notification(data));
            }
            "revocation" => {
                eprintln!("Subscription revoked: {}", data["payload"]["subscription"]);
            }
            _ => {}
        }
    }
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().trim().to_string()
}

async fn handle_notification(data: Value) {
    let event = &data["payload"]["event"];

    match data["metadata"]["subscription_type"]
        .as_str()
        .unwrap_or_default()
    {
        "channel.chat.message" => {
            let user_messager = text(&event["chatter_user_login"]);
            let messager_id = text(&event["chatter_user_id"]);
            let message = text(&event["message"]["text"]);

            if handle_auto_bans(&user_messager, &messager_id, &message).await {
                return;
            }
            shipping_users(&user_messager, &message).await;
            simple_response_commands(&user_messager, &message).await;
            handle_dad_jokes(&user_messager, &message).await;
        }
        "channel.channel_points_custom_reward_redemption.add" => {
            let redeem = text(&event["reward"]["title"]);
            let user = text(&event["user_login"]);
            let user_id = text(&event["user_id"]);

            handle_redeems(&redeem, &user, &user_id).await;
        }
        "channel.raid" => {
            if text(&event["to_broadcaster_user_id"]) != USER_ID {
                return;
            }
            let raider_user_name = text(&event["from_broadcaster_user_name"]);
            let raider_user_id = text(&event["from_broadcaster_user_id"]);

            handle_raids(&raider_user_name, &raider_user_id).await;
        }
        _ => {}
    }
}
