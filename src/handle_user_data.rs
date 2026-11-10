use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::constants::DATABASE_FILEPATH;
use crate::types::User;

fn open() -> rusqlite::Result<Connection> {
    Connection::open(DATABASE_FILEPATH.as_str())
}

/// Creates the database file and the users table if they don't exist yet.
pub fn init_database() -> rusqlite::Result<()> {
    if let Some(parent) = Path::new(DATABASE_FILEPATH.as_str()).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    open()?.execute_batch(include_str!("../database/schema.sql"))
}

pub fn get_user(user_id: &str) -> rusqlite::Result<Option<User>> {
    open()?
        .query_row(
            "SELECT user_id, first, second, third, daily FROM users WHERE user_id = ?",
            [user_id],
            |row| {
                Ok(User {
                    user_id: row.get(0)?,
                    first: row.get(1)?,
                    second: row.get(2)?,
                    third: row.get(3)?,
                    daily: row.get(4)?,
                })
            },
        )
        .optional()
}

pub fn create_user(user: &User) -> rusqlite::Result<()> {
    open()?.execute(
        "INSERT INTO users (user_id, first, second, third, daily) VALUES (?, ?, ?, ?, ?)",
        params![
            user.user_id,
            user.first,
            user.second,
            user.third,
            user.daily
        ],
    )?;
    Ok(())
}

pub fn update_user(user: &User) -> rusqlite::Result<()> {
    open()?.execute(
        "UPDATE users SET first = ?, second = ?, third = ?, daily = ? WHERE user_id = ?",
        params![
            user.first,
            user.second,
            user.third,
            user.daily,
            user.user_id
        ],
    )?;
    Ok(())
}
