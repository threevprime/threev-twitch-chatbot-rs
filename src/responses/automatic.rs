use crate::constants::{BANNABLE_SENTENCES, STREAMER_USERNAME};
use crate::requests::{ban_user, send_chat_message, send_shout_out};

const I_AM_ARRAY: [&str; 8] = ["I am", "Im", "I'm", "I’m", "i am", "im", "i'm", "i’m"];

pub async fn handle_raids(raider_user_name: &str, raider_user_id: &str) {
    send_chat_message(&format!("!so {raider_user_name}")).await;
    send_shout_out(raider_user_id).await;
}

pub async fn handle_dad_jokes(user_messager: &str, message: &str) {
    let name = STREAMER_USERNAME.clone();
    if user_messager.to_lowercase() == name {
        return;
    }

    for i_am in I_AM_ARRAY {
        if let Some(reply) = find_i_am(message, i_am) {
            send_chat_message(&format!("Hi {reply}. I'm {name}.")).await;
            return;
        }
    }
}

/// Finds "i am" as its own word (so "Imagine" doesn't count) and returns
/// whatever comes after it.
fn find_i_am<'a>(message: &'a str, i_am: &str) -> Option<&'a str> {
    for (index, _) in message.match_indices(i_am) {
        let starts_word = index == 0 || message.as_bytes()[index - 1] == b' ';
        let rest = &message[index + i_am.len()..];
        let ends_word = rest.is_empty() || rest.starts_with(' ');
        let reply = rest.trim();

        if starts_word && ends_word && !reply.is_empty() {
            return Some(reply);
        }
    }
    None
}

/// Returns true if the user got banned, so the other handlers can skip them.
pub async fn handle_auto_bans(user_messager: &str, user_id: &str, message: &str) -> bool {
    let name = STREAMER_USERNAME.clone();
    if user_messager.to_lowercase() == name {
        return false;
    }

    for bannable_sentence in BANNABLE_SENTENCES {
        if message.contains(bannable_sentence) {
            ban_user(user_id, "Spam / bot message").await;
            send_chat_message(&format!("{user_messager} has been banned!")).await;
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::find_i_am;

    #[test]
    fn dad_joke_matching() {
        assert_eq!(find_i_am("I'm tired", "I'm"), Some("tired"));
        assert_eq!(find_i_am("ok im hungry now", "im"), Some("hungry now"));
        assert_eq!(find_i_am("I’m on mobile", "I’m"), Some("on mobile"));
        assert_eq!(find_i_am("Imagine that", "Im"), None);
        assert_eq!(find_i_am("swimming time", "im"), None);
        assert_eq!(find_i_am("im", "im"), None);
    }
}
