use std::sync::LazyLock;

use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde_json::{Value, json};

use crate::constants::{CLIENT_ID, OAUTH_TOKEN, USER_ID};

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

fn helix(method: Method, endpoint: &str) -> RequestBuilder {
    CLIENT
        .request(method, format!("https://api.twitch.tv/helix/{endpoint}"))
        .bearer_auth(OAUTH_TOKEN.as_str())
        .header("Client-Id", CLIENT_ID.as_str())
}

/// Twitch requires validating the token on startup and every hour after that.
/// Exits the process if Twitch says the token is no longer valid.
pub async fn get_auth() {
    let response = CLIENT
        .get("https://id.twitch.tv/oauth2/validate")
        .header("Authorization", format!("OAuth {}", OAUTH_TOKEN.as_str()))
        .send()
        .await;

    match response {
        Ok(response) if response.status() == StatusCode::OK => println!("Validated token."),
        Ok(response) => {
            eprintln!(
                "Token is not valid. /oauth2/validate returned status code {}",
                response.status()
            );
            eprintln!("{}", response.text().await.unwrap_or_default());
            std::process::exit(1);
        }
        Err(error) => eprintln!("Could not reach /oauth2/validate: {error}"),
    }
}

pub async fn register_event_sub_listener(
    websocket_session_id: &str,
    subscription_type: &str,
    version: &str,
    condition: Value,
) {
    let response = helix(Method::POST, "eventsub/subscriptions")
        .json(&json!({
            "type": subscription_type,
            "version": version,
            "condition": condition,
            "transport": {
                "method": "websocket",
                "session_id": websocket_session_id,
            },
        }))
        .send()
        .await;

    match response {
        Ok(response) if response.status() == StatusCode::ACCEPTED => {
            let data: Value = response.json().await.unwrap_or_default();
            println!(
                "Subscribed to {subscription_type} [{}]",
                data["data"][0]["id"].as_str().unwrap_or("?")
            );
        }
        Ok(response) => {
            eprintln!(
                "Failed to subscribe to {subscription_type}. API call returned status code {}",
                response.status()
            );
            eprintln!("{}", response.text().await.unwrap_or_default());
            std::process::exit(1);
        }
        Err(error) => eprintln!("{error}"),
    }
}

/// `to` must be the user ID of the channel, not the username.
pub async fn send_shout_out(to: &str) {
    let response = helix(Method::POST, "chat/shoutouts")
        .query(&[
            ("from_broadcaster_id", USER_ID.as_str()),
            ("to_broadcaster_id", to),
            ("moderator_id", USER_ID.as_str()),
        ])
        .send()
        .await;

    match response {
        Ok(response) if response.status() == StatusCode::NO_CONTENT => {
            println!("Shouted out {to}")
        }
        Ok(response) => {
            eprintln!("Failed to send shoutout");
            eprintln!("{}", response.text().await.unwrap_or_default());
        }
        Err(error) => eprintln!("{error}"),
    }
}

pub async fn send_chat_message(chat_message: &str) {
    let response = helix(Method::POST, "chat/messages")
        .json(&json!({
            "broadcaster_id": USER_ID.as_str(),
            "sender_id": USER_ID.as_str(),
            "message": chat_message,
        }))
        .send()
        .await;

    match response {
        Ok(response) if response.status() == StatusCode::OK => {
            // A 200 doesn't mean it was posted, AutoMod etc. can still drop it
            let data: Value = response.json().await.unwrap_or_default();
            if data["data"][0]["is_sent"].as_bool() == Some(true) {
                println!("Sent chat message: {chat_message}");
            } else {
                eprintln!(
                    "Chat message was dropped: {}",
                    data["data"][0]["drop_reason"]
                );
            }
        }
        Ok(response) => {
            eprintln!("Failed to send chat message");
            eprintln!("{}", response.text().await.unwrap_or_default());
        }
        Err(error) => eprintln!("{error}"),
    }
}

pub async fn ban_user(user_id: &str, reason: &str) {
    let response = helix(Method::POST, "moderation/bans")
        .query(&[
            ("broadcaster_id", USER_ID.as_str()),
            ("moderator_id", USER_ID.as_str()),
        ])
        .json(&json!({
            "data": {
                "user_id": user_id,
                "reason": reason,
            },
        }))
        .send()
        .await;

    match response {
        Ok(response) if response.status() == StatusCode::OK => println!("Banned {user_id}"),
        Ok(response) => {
            eprintln!("Failed to ban user");
            eprintln!("{}", response.text().await.unwrap_or_default());
        }
        Err(error) => eprintln!("{error}"),
    }
}
