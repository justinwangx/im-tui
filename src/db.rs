use crate::error::{Error, Result};
use chrono::{DateTime, Local, TimeZone};
use rusqlite::{params, Connection};
use std::env;
use std::path::PathBuf;

/// Database path relative to the home directory.
const DB_PATH: &str = "Library/Messages/chat.db";

/// Struct representing the Messages database.
pub struct MessageDB {
    conn: Connection,
}

impl MessageDB {
    /// Open the Messages database.
    pub fn open() -> Result<Self> {
        // Build the path to the Messages database
        let home_dir = env::var("HOME")?;
        let mut db_path = PathBuf::from(home_dir);
        db_path.push(DB_PATH);

        // Open the database
        let conn = Connection::open(db_path)?;

        Ok(Self { conn })
    }

    /// Get messages for a contact.
    pub fn get_messages(
        &self,
        contact: &str,
    ) -> Result<Vec<(Option<String>, DateTime<Local>, Option<String>, bool)>> {
        // SQL query to select messages FROM the specified contact (not TO them)
        let query = r#"
            SELECT text,
                   date / 1000000000 + strftime('%s','2001-01-01') as unix_timestamp,
                   CASE
                       WHEN is_audio_message = 1 THEN 'Audio Message'
                       WHEN cache_has_attachments = 1 AND (text IS NULL OR text = '￼') THEN 'Image'
                       WHEN balloon_bundle_id IS NOT NULL THEN 'iMessage Effect'
                       WHEN item_type != 0 THEN 'Special Message'
                       ELSE NULL
                   END as message_type,
                   is_from_me
            FROM message
            JOIN handle ON message.handle_id = handle.ROWID
            WHERE handle.id = ?
            ORDER BY date DESC
            LIMIT 50;
        "#;

        let mut stmt = self.conn.prepare(query)?;
        let mut rows = stmt.query(params![contact])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            // Retrieve the text and timestamp for the message
            let text: Option<String> = row.get(0)?;
            let timestamp: i64 = row.get(1)?;
            let message_type: Option<String> = row.get(2)?;
            let is_from_me: bool = row.get(3)?;

            // Convert Unix timestamp to DateTime<Local>
            let dt = match Local.timestamp_opt(timestamp, 0) {
                chrono::LocalResult::Single(dt) => dt,
                _ => return Err(Error::Generic("Invalid timestamp".to_string())),
            };

            messages.push((text, dt, message_type, is_from_me));
        }

        Ok(messages)
    }
}
