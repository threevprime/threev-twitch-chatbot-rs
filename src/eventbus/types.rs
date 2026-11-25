use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum EventType {
    Message(MessageEvent),
    Raid(RaidEvent),
}

#[derive(Clone, Debug, Serialize)]
pub struct MessageEvent {
    pub user_id: String,
    pub username: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RaidEvent {
    pub user_id: String,
    pub username: String,
    pub viewers: u32,
}
