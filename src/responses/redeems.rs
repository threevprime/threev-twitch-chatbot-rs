use crate::bot::TwitchBot;
use crate::handle_user_data::{create_user, get_user, update_user};
use crate::types::User;

pub async fn handle_redeems(bot: &TwitchBot, redeem: &str, user: &str, user_id: &str) {
    match redeem {
        "first" | "second" | "third" => {
            let Some(times) = increment_redeem(user_id, redeem) else {
                return;
            };
            bot.send_chat_message(&format!(
                "{user} is the {redeem} person to join the stream for the {} time!",
                times_redeemed(times)
            ))
            .await;
        }
        "daily" => {
            let Some(times) = increment_redeem(user_id, redeem) else {
                return;
            };
            bot.send_chat_message(&format!(
                "{user} has joined the stream for the {times} time!"
            ))
            .await;
        }
        _ => {
            bot.send_chat_message(&format!("{redeem} has been redeemed by {user}!"))
                .await
        }
    }
}

/// Adds 1 to the user's counter for this redeem (creating the user if needed)
/// and returns the new count.
fn increment_redeem(user_id: &str, redeem: &str) -> Option<i64> {
    match try_increment_redeem(user_id, redeem) {
        Ok(times) => Some(times),
        Err(error) => {
            eprintln!("Database error for {user_id}: {error}");
            None
        }
    }
}

fn try_increment_redeem(user_id: &str, redeem: &str) -> rusqlite::Result<i64> {
    let existing = get_user(user_id)?;
    let is_new = existing.is_none();
    let mut user = existing.unwrap_or(User {
        user_id: user_id.to_string(),
        first: 0,
        second: 0,
        third: 0,
        daily: 0,
    });

    let counter = match redeem {
        "first" => &mut user.first,
        "second" => &mut user.second,
        "third" => &mut user.third,
        _ => &mut user.daily,
    };
    *counter += 1;
    let times = *counter;

    if is_new {
        create_user(&user)?;
    } else {
        update_user(&user)?;
    }
    Ok(times)
}

fn times_redeemed(times: i64) -> String {
    match times {
        1 => "first".to_string(),
        2 => "second".to_string(),
        3 => "third".to_string(),
        _ => format!("{times}th"),
    }
}
