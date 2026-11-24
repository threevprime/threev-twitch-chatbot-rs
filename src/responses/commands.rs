use crate::bot::TwitchBot;
use crate::utils::random_percentage;

pub async fn shipping_users(bot: &TwitchBot, user: &str, other: &str) {
    bot.send_chat_message(&format!(
        "{user} and {other} are {}% in love!",
        random_percentage()
    ))
    .await
}
