use serde_json::json;

use log::debug;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::Value;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::bot::TwitchBot;
use crate::constants::EVENTSUB_WEBSOCKET_URL;
use crate::responses::automatic::{handle_auto_bans, handle_dad_jokes, handle_raids};
use crate::responses::commands::{shipping_users, simple_response_commands};
use crate::responses::redeems::handle_redeems;

async fn handle_registering_event_sub_listeners(bot: &TwitchBot, websocket_session_id: &str) {
    bot.register_event_sub_listener(
        websocket_session_id,
        "channel.chat.message",
        "1",
        json!({
            "broadcaster_user_id": bot.user_id(),
            "user_id": bot.user_id(),
        }),
    )
    .await;
    bot.register_event_sub_listener(
        websocket_session_id,
        "channel.channel_points_custom_reward_redemption.add",
        "1",
        json!({
            "broadcaster_user_id": bot.user_id(),
        }),
    )
    .await;
    bot.register_event_sub_listener(
        websocket_session_id,
        "channel.raid",
        "1",
        json!({
            "to_broadcaster_user_id": bot.user_id(),
        }),
    )
    .await;
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().trim().to_string()
}

async fn handle_notification(bot: Arc<TwitchBot>, data: Value) {
    let event = &data["payload"]["event"];

    match data["metadata"]["subscription_type"]
        .as_str()
        .unwrap_or_default()
    {
        "channel.chat.message" => {
            let user_messager = text(&event["chatter_user_login"]);
            let messager_id = text(&event["chatter_user_id"]);
            let message = text(&event["message"]["text"]);
            debug!("{user_messager}({messager_id}): {message}");

            if handle_auto_bans(&bot, &user_messager, &messager_id, &message).await {
                return;
            }
            shipping_users(&bot, &user_messager, &message).await;
            simple_response_commands(&bot, &user_messager, &message).await;
            handle_dad_jokes(&bot, &user_messager, &message).await;
        }
        "channel.channel_points_custom_reward_redemption.add" => {
            let redeem_name = text(&event["reward"]["title"]);
            let user = text(&event["user_login"]);
            let user_id = text(&event["user_id"]);
            debug!("{user}({user_id}) redeemed {redeem_name}");

            handle_redeems(&bot, &redeem_name, &user, &user_id).await;
        }
        "channel.raid" => {
            let raider_user_name = text(&event["from_broadcaster_user_name"]);
            let raider_user_id = text(&event["from_broadcaster_user_id"]);
            debug!("{raider_user_name}({raider_user_id}) raided");
            if text(&event["to_broadcaster_user_id"]) != bot.user_id() {
                return;
            }

            handle_raids(&bot, &raider_user_name, &raider_user_id).await;
        }
        _ => {}
    }
}

pub async fn start_web_socket_client(bot: Arc<TwitchBot>) -> Result<(), Box<dyn Error>> {
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
                    handle_registering_event_sub_listeners(&bot, websocket_session_id).await;
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
                tokio::spawn(handle_notification(bot.clone(), data));
            }
            "revocation" => {
                eprintln!("Subscription revoked: {}", data["payload"]["subscription"]);
            }
            _ => {}
        }
    }
}
