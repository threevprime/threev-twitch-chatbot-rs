use crate::constants::API_BASE_URL;
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde_json::{Value, json};

pub struct TwitchBot {
    token: String,
    client_id: String,
    user_id: String,
    client: Client,
}

impl TwitchBot {
    pub fn new(token: String, client_id: String, user_id: String) -> Self {
        Self {
            token: token,
            client_id: client_id,
            user_id: user_id,
            client: Client::new(),
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    fn helix(&self, method: Method, endpoint: &str) -> RequestBuilder {
        self.client
            .request(method, format!("{API_BASE_URL}/{endpoint}"))
            .bearer_auth(self.token.as_str())
            .header("Client-Id", self.client_id.as_str())
    }

    pub async fn get_auth(&self) {
        let response = self
            .client
            .get("https://id.twitch.tv/oauth2/validate")
            .header("Authorization", format!("OAuth {}", &self.token))
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

    pub async fn send_chat_message(&self, chat_message: &str) {
        let response = self
            .helix(Method::POST, "chat/messages")
            .json(&json!({
                "broadcaster_id": self.user_id,
                "sender_id": self.user_id,
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

    pub async fn send_shout_out(&self, to: &str) {
        let response = self
            .helix(Method::POST, "chat/shoutouts")
            .query(&[
                ("from_broadcaster_id", self.user_id.as_str()),
                ("to_broadcaster_id", to),
                ("moderator_id", self.user_id.as_str()),
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

    pub async fn ban_user(&self, user_id: &str, reason: &str) {
        let response = self
            .helix(Method::POST, "moderation/bans")
            .query(&[
                ("broadcaster_id", &self.user_id),
                ("moderator_id", &self.user_id),
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

    pub async fn register_event_sub_listener(
        &self,
        websocket_session_id: &str,
        subscription_type: &str,
        version: &str,
        condition: Value,
    ) {
        let response = self
            .helix(Method::POST, "eventsub/subscriptions")
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
}
