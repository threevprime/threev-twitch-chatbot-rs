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
