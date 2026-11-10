use serde_json::json;

use crate::constants::USER_ID;
use crate::requests::register_event_sub_listener;

pub async fn handle_registering_event_sub_listeners(websocket_session_id: &str) {
    register_event_sub_listener(
        websocket_session_id,
        "channel.chat.message",
        "1",
        json!({
            "broadcaster_user_id": USER_ID,
            "user_id": USER_ID,
        }),
    )
    .await;
    register_event_sub_listener(
        websocket_session_id,
        "channel.channel_points_custom_reward_redemption.add",
        "1",
        json!({
            "broadcaster_user_id": USER_ID,
        }),
    )
    .await;
    register_event_sub_listener(
        websocket_session_id,
        "channel.raid",
        "1",
        json!({
            "to_broadcaster_user_id": USER_ID,
        }),
    )
    .await;
}
