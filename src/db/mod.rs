pub mod commands;
pub mod users;
pub mod widgets;

use std::path::Path;

use rusqlite::Connection;

use crate::constants::DATABASE_FILEPATH;

fn open() -> rusqlite::Result<Connection> {
    Connection::open(DATABASE_FILEPATH.as_str())
}

/// Creates the database file and the users table if they don't exist yet.
pub fn init_database() -> rusqlite::Result<()> {
    if let Some(parent) = Path::new(DATABASE_FILEPATH.as_str()).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    open()?.execute_batch(include_str!("../../database/schema.sql"))
}
