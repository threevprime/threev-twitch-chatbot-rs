use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use log::{error, info, warn};
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde_json::{Value, json};
use tokio::time::interval;

use crate::constants::API_BASE_URL;
use crate::types::Command;

pub struct TwitchBot {
    token: String,
    client_id: String,
    user_id: String,
    client: Client,
    commands: RwLock<HashMap<String, Arc<Command>>>,
}

impl TwitchBot {
    pub fn new(token: String, client_id: String, user_id: String) -> Arc<Self> {
        let bot = Arc::new(Self {
            token: token,
            client_id: client_id,
            user_id: user_id,
            client: Client::new(),
            commands: RwLock::new(HashMap::new()),
        });

        let validator = Arc::clone(&bot);
        tokio::spawn(async move {
            validator.get_auth().await;
            let mut hourly = interval(Duration::from_secs(60 * 60));
            hourly.tick().await;
            loop {
                hourly.tick().await;
                validator.get_auth().await;
            }
        });

        bot
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn commands(&self) -> Vec<Arc<Command>> {
        let comamnds = self.commands.read().unwrap_or_else(|e| e.into_inner());
        let mut unique_commands: Vec<Arc<Command>> = Vec::new();

        for cmd in comamnds.values() {
            if !unique_commands
                .iter()
                .any(|existing| Arc::ptr_eq(existing, cmd))
            {
                unique_commands.push(Arc::clone(cmd));
            }
        }

        unique_commands
    }

    pub fn get_command(&self, name: &str) -> Option<Arc<Command>> {
        self.commands.read().unwrap().get(name).cloned()
    }

    pub fn add_command(&self, command: Command) {
        let command = Arc::new(command);
        let mut commands = self.commands.write().unwrap();

        if let Some(alias) = command.alias() {
            commands.insert(alias.to_string(), Arc::clone(&command));
        }

        commands.insert(command.name().to_string(), command);
    }

    pub fn remove_command(&self, name: &str) {
        self.commands.write().unwrap().remove(name);
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
            Ok(response) if response.status() == StatusCode::OK => info!("Validated token."),
            Ok(response) => {
                error!(
                    "Token is not valid. /oauth2/validate returned status code {}",
                    response.status()
                );
                error!("{}", response.text().await.unwrap_or_default());
                std::process::exit(1);
            }
            Err(error) => error!("Could not reach /oauth2/validate: {error}"),
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
                    info!("Sent chat message: {chat_message}");
                } else {
                    warn!(
                        "Chat message was dropped: {}",
                        data["data"][0]["drop_reason"]
                    );
                }
            }
            Ok(response) => {
                error!("Failed to send chat message");
                error!("{}", response.text().await.unwrap_or_default());
            }
            Err(error) => error!("{error}"),
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
                info!("Shouted out {to}")
            }
            Ok(response) => {
                error!("Failed to send shoutout");
                error!("{}", response.text().await.unwrap_or_default());
            }
            Err(error) => error!("{error}"),
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
            Ok(response) if response.status() == StatusCode::OK => info!("Banned {user_id}"),
            Ok(response) => {
                error!("Failed to ban user");
                error!("{}", response.text().await.unwrap_or_default());
            }
            Err(error) => error!("{error}"),
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
                info!(
                    "Subscribed to {subscription_type} [{}]",
                    data["data"][0]["id"].as_str().unwrap_or("?")
                );
            }
            Ok(response) => {
                error!(
                    "Failed to subscribe to {subscription_type}. API call returned status code {}",
                    response.status()
                );
                error!("{}", response.text().await.unwrap_or_default());
                std::process::exit(1);
            }
            Err(error) => error!("{error}"),
        }
    }
}
