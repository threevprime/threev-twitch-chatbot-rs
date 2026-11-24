use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AddCommandRequest {
    pub name: String,
    pub alias: Option<String>,
    pub response: String,
}

#[derive(Serialize)]
pub struct UploadWidgetResponse {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct WidgetResponse {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}
