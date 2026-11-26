use rusqlite::params;

use super::open;
use crate::types::Command;

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
