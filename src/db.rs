use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::constants::DATABASE_FILEPATH;
use crate::types::{Command, User, Widget};

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

pub fn get_commands() -> rusqlite::Result<Vec<Command>> {
    let conn = open()?;

    let mut stmt = conn.prepare("SELECT * FROM commands")?;
    let commands = stmt.query_map([], |row| {
        Ok(Command::new(row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    commands.collect()
}

pub fn create_command(name: &str, alias: Option<&str>, response: &str) -> rusqlite::Result<()> {
    let conn = open()?;

    conn.execute(
        "INSERT INTO commands (name, alias, response) VALUES (?, ?, ?)",
        params![name, alias, response],
    )?;

    Ok(())
}

pub fn remove_command(name: &str) -> rusqlite::Result<bool> {
    let conn = open()?;
    let affected = conn.execute("DELETE FROM commands WHERE name = ?", [name])?;
    Ok(affected > 0)
}

pub fn get_widgets() -> rusqlite::Result<Vec<Widget>> {
    let conn = open()?;

    let mut stmt = conn.prepare("SELECT * FROM widgets")?;
    let commands = stmt.query_map([], |row| {
        Ok(Widget {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    commands.collect()
}

pub fn create_widget(name: &str) -> rusqlite::Result<i64> {
    let conn = open()?;
    conn.execute("INSERT INTO widgets (name) VALUES (?)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn has_widget(id: i64) -> rusqlite::Result<bool> {
    let conn = open()?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM widgets WHERE id = ?",
        params![id],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

pub fn remove_widget(id: i64) -> rusqlite::Result<bool> {
    let conn = open()?;
    let affected = conn.execute("DELETE FROM widgets WHERE id = ?", [id])?;
    Ok(affected > 0)
}
