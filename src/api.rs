use axum::{Json, Router, routing::get};
use std::sync::Arc;

use crate::bot::TwitchBot;

pub fn create_router(bot: Arc<TwitchBot>) -> Router {
    Router::new().route("/health", get(health)).with_state(bot)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
}
