use crate::bot::TwitchBot;
use crate::utils::random_percentage;

pub async fn simple_response_commands(bot: &TwitchBot, user_messager: &str, message: &str) {
    match message.to_lowercase().as_str() {
        "bug" => bot.send_chat_message("threev6Smokinghead").await,
        "!help" => {
            bot.send_chat_message(
                "!github - Gives github URL, !ship - Shows the compatability of 2 users, !sussy - Shows how sussy a person is, !rickrolled - yes",
            )
            .await
        }
        "!github" => bot.send_chat_message("https://github.com/threevprime").await,
        "!rickrolled" => bot.send_chat_message("https://www.youtube.com/watch?v=dQw4w9WgXcQ").await,
        "!sussy" => {
            let sus_percentage = random_percentage();
            bot.send_chat_message(&format!("@{user_messager} is {sus_percentage}% sussy!")).await
        }
        "!raid" => send_raid_messages(&bot).await,
        _ => {}
    }
}

pub async fn shipping_users(bot: &TwitchBot, user: &str, other: &str) {
    bot.send_chat_message(&format!(
        "{user} and {other} are {}% in love!",
        random_percentage()
    ))
    .await
}

pub async fn send_raid_messages(bot: &TwitchBot) {
    bot.send_chat_message(
        r#"if stream_status(Threev) == "offline" {    println!("Raiding a channel!");    raid();}"#,
    )
    .await;
    bot.send_chat_message(
        r#"threev6Smokinghead if stream_status(Threev) == "offline" { println!("Raiding a channel!"); raid();} threev6Smokinghead"#,
    )
    .await;
}
