use rusqlite::params;

use super::open;
use crate::types::Widget;

pub fn get_widgets() -> rusqlite::Result<Vec<Widget>> {
    let conn = open()?;

    let mut stmt = conn.prepare("SELECT * FROM widgets")?;
    let widgets = stmt.query_map([], |row| {
        Ok(Widget {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    widgets.collect()
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
