use axum::{
    Json,
    body::Body,
    extract::{
        Multipart, Path, State,
        ws::{Message, WebSocketUpgrade},
    },
    http::{HeaderValue, StatusCode, header},
    response::{Html, Response},
};
use log::error;
use tokio::fs;
use zip::ZipArchive;

use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use crate::bot::TwitchBot;
use crate::eventbus::EventType;
use crate::db;
use crate::types::Command;

use super::types::{AddCommandRequest, CommandResponse, UploadWidgetResponse, WidgetResponse};

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn websocket(State(bot): State<Arc<TwitchBot>>, ws: WebSocketUpgrade) -> Response {
    let mut receiver = bot.eventbus().subscribe();
    ws.on_upgrade(|mut socket| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let message = match event {
                        EventType::Message(data) => {
                            serde_json::json!({"type": "message", "data": data})
                        }
                        EventType::Raid(data) => serde_json::json!({"type": "raid", "data": data}),
                    };
                    if let Err(e) = socket.send(Message::Text(message.to_string().into())).await {
                        error!("Error sending data over ws: {:?}", e);
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

pub async fn get_commands(State(bot): State<Arc<TwitchBot>>) -> Json<Vec<CommandResponse>> {
    Json(
        bot.commands()
            .iter()
            .map(|command| CommandResponse {
                name: command.name().to_string(),
                alias: command.alias().map(|s| s.to_string()),
                response: command.response().to_string(),
            })
            .collect(),
    )
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

    match db::commands::create_command(&name, alias.as_deref(), &response) {
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
    match db::commands::remove_command(&name) {
        Ok(_) => {
            bot.remove_command(&name);
            StatusCode::NO_CONTENT
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn get_all_widgets() -> Result<Json<Vec<WidgetResponse>>, StatusCode> {
    let widgets = db::widgets::get_widgets().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
        db::widgets::create_widget(&name).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    archive
        .extract(format!("./data/widgets/{id}"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    return Ok(Json(UploadWidgetResponse {
        id: id.to_string(),
        name: name,
    }));
}

pub async fn get_widget(Path(id): Path<i64>) -> Result<Html<String>, StatusCode> {
    if !db::widgets::has_widget(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
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
    if !db::widgets::has_widget(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
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
    if !db::widgets::has_widget(id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::BAD_REQUEST);
    }

    let path = PathBuf::from(format!("./data/widgets/{id}"));
    if !path.exists() {
        return Ok(StatusCode::BAD_REQUEST);
    }

    if !db::widgets::remove_widget(id).unwrap() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    fs::remove_dir_all(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}
