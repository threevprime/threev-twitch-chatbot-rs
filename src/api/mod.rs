mod endpoints;
mod types;

use axum::{
    Router,
    response::Redirect,
    routing::{delete, get, post},
};
use std::sync::Arc;

use crate::bot::TwitchBot;

pub fn create_router(bot: Arc<TwitchBot>) -> Router {
    Router::new()
        .route("/health", get(endpoints::health))
        // Custom Commands Endpoints
        .route("/commands", get(endpoints::get_commands))
        .route("/command/create", post(endpoints::add_command))
        .route("/command/{name}", delete(endpoints::remove_command))
        // Widget Endpoints
        .route("/widgets", get(endpoints::get_all_widgets))
        .route("/widget/upload", post(endpoints::upload_widget))
        .route(
            "/widget/{id}",
            get(
                |axum::extract::Path(id): axum::extract::Path<String>| async move {
                    Redirect::permanent(&format!("/widget/{id}/"))
                },
            ),
        )
        .route("/widget/{id}/", get(endpoints::get_widget))
        .route("/widget/{id}/{*asset}", get(endpoints::get_widget_asset))
        .route("/widget/{id}", delete(endpoints::delete_widget))
        .with_state(bot)
}
