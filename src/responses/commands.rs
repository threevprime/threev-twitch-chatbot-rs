use crate::requests::send_chat_message;
use crate::utils::random_percentage;

pub async fn simple_response_commands(user_messager: &str, message: &str) {
    match message.to_lowercase().as_str() {
        "bug" => send_chat_message("threev6Smokinghead").await,
        "!help" => {
            send_chat_message(
                "!github - Gives github URL, !ship - Shows the compatability of 2 users, !sussy - Shows how sussy a person is, !rickrolled - yes",
            )
            .await
        }
        "!github" => send_chat_message("https://github.com/threevprime").await,
        "!rickrolled" => send_chat_message("https://www.youtube.com/watch?v=dQw4w9WgXcQ").await,
        "!sussy" => {
            let sus_percentage = random_percentage();
            send_chat_message(&format!("@{user_messager} is {sus_percentage}% sussy!")).await
        }
        "!raid" => send_raid_messages().await,
        _ => {}
    }
}

pub async fn shipping_users(user_messager: &str, message: &str) {
    let message_array: Vec<&str> = message.split_whitespace().collect();
    if message_array.first() != Some(&"!ship") {
        return;
    }

    match message_array.as_slice() {
        [_] => send_chat_message("Usage: !ship [username_1] [username_2]").await,
        [_, first] => {
            send_chat_message(&format!(
                "{first} and {user_messager} are {}% in love!",
                random_percentage()
            ))
            .await
        }
        [_, first, second, ..] => {
            send_chat_message(&format!(
                "{first} and {second} are {}% in love!",
                random_percentage()
            ))
            .await
        }
        [] => {}
    }
}

pub async fn send_raid_messages() {
    send_chat_message(
        r#"if stream_status(Threev) == "offline" {    println!("Raiding a channel!");    raid();}"#,
    )
    .await;
    send_chat_message(
        r#"threev6Smokinghead if stream_status(Threev) == "offline" { println!("Raiding a channel!"); raid();} threev6Smokinghead"#,
    )
    .await;
}
