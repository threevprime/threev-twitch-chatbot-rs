use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::bot::TwitchBot;
use crate::handle_user_data;
use crate::types::Command;

#[derive(Deserialize)]
pub struct AddCommandRequest {
    pub name: String,
    pub alias: Option<String>,
    pub response: String,
}

pub fn create_router(bot: Arc<TwitchBot>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/commands", get(get_commands))
        .route("/command", post(add_command))
        .route("/command/{name}", delete(remove_command))
        .with_state(bot)
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn get_commands(State(bot): State<Arc<TwitchBot>>) -> Json<serde_json::Value> {
    Json(serde_json::json!(
        bot.commands()
            .iter()
            .map(|command| {
                serde_json::json!({
                    "name": command.name(),
                    "alias": command.alias(),
                    "response": command.response()
                })
            })
            .collect::<Vec<_>>()
    ))
}

async fn add_command(
    State(bot): State<Arc<TwitchBot>>,
    Json(payload): Json<AddCommandRequest>,
) -> StatusCode {
    let AddCommandRequest {
        name,
        alias,
        response,
    } = payload;

    match handle_user_data::create_command(&name, alias.as_deref(), &response) {
        Ok(_) => {
            bot.add_command(Command::new(name, alias, response));
            StatusCode::CREATED
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn remove_command(State(bot): State<Arc<TwitchBot>>, Path(name): Path<String>) -> StatusCode {
    match handle_user_data::remove_command(&name) {
        Ok(_) => {
            bot.remove_command(&name);
            StatusCode::NO_CONTENT
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
