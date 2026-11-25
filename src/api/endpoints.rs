use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderValue, StatusCode, header},
    response::Html,
    response::Response,
};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use zip::ZipArchive;

use super::types::{AddCommandRequest, UploadWidgetResponse, WidgetResponse};
use crate::bot::TwitchBot;
use crate::handle_user_data;
use crate::types::Command;

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn websocket(State(bot): State<Arc<TwitchBot>>, ws: WebSocketUpgrade) -> Response {
    let mut receiver = bot.eventbus().subscribe();
    ws.on_upgrade(|mut socket| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let json = match serde_json::to_string(&event) {
                        Ok(json) => json,
                        Err(e) => {
                            error!("Serialization error: {:?}", e);
                            continue;
                        }
                    };

                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Event receiver error: {:?}", e);
                    break;
                }
            }
        }
    })
}

pub async fn get_commands(State(bot): State<Arc<TwitchBot>>) -> Json<serde_json::Value> {
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

pub async fn add_command(
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

pub async fn remove_command(
    State(bot): State<Arc<TwitchBot>>,
    Path(name): Path<String>,
) -> StatusCode {
    match handle_user_data::remove_command(&name) {
        Ok(_) => {
            bot.remove_command(&name);
            StatusCode::NO_CONTENT
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn get_all_widgets() -> Result<Json<Vec<WidgetResponse>>, StatusCode> {
    let widgets = handle_user_data::get_widgets().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        widgets
            .iter()
            .map(|w| WidgetResponse {
                id: w.id,
                name: w.name.clone(),
                created_at: w.created_at.clone(),
            })
            .collect(),
    ))
}

pub async fn upload_widget(
    mut multipart: Multipart,
) -> Result<Json<UploadWidgetResponse>, StatusCode> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let name = field
        .file_name()
        .and_then(|f| std::path::Path::new(f).file_stem())
        .and_then(|s| s.to_str())
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();
    let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut archive = ZipArchive::new(Cursor::new(data)).map_err(|_| StatusCode::BAD_REQUEST)?;
    if !archive.file_names().any(|name| name == "index.html") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id =
        handle_user_data::create_widget(&name).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    archive
        .extract(format!("./data/widgets/{id}"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    return Ok(Json(UploadWidgetResponse {
        id: id.to_string(),
        name: name,
    }));
}

pub async fn get_widget(Path(id): Path<i64>) -> Result<Html<String>, StatusCode> {
    if !handle_user_data::has_widget(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut path = PathBuf::from(format!("./data/widgets/{id}"));
    if !path.exists() {
        return Err(StatusCode::BAD_REQUEST);
    }
    path.push("index.html");
    if !path.exists() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Html(fs::read_to_string(path).await.unwrap()))
}

pub async fn get_widget_asset(
    Path((id, asset)): Path<(i64, String)>,
) -> Result<Response, StatusCode> {
    if !handle_user_data::has_widget(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut path = PathBuf::from(format!("./data/widgets/{id}"));
    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
    path.push(asset);
    if !path.exists() || !path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let content = fs::read(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let content_type = mime_guess::from_path(&path).first_or_octet_stream();

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type.as_ref()).unwrap(),
        )
        .body(Body::from(content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn delete_widget(Path(id): Path<i64>) -> Result<StatusCode, StatusCode> {
    if !handle_user_data::has_widget(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::BAD_REQUEST);
    }

    let path = PathBuf::from(format!("./data/widgets/{id}"));
    if !path.exists() {
        return Ok(StatusCode::BAD_REQUEST);
    }

    if !handle_user_data::remove_widget(id).unwrap() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    fs::remove_dir_all(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}
