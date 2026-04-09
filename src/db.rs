use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use sqlite_vec::sqlite3_vec_init;
use rusqlite::ffi::sqlite3_auto_extension;
use zerocopy::AsBytes;

pub struct Thought {
    pub id: i64,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: String,
}

pub struct Store {
    pub conn: Connection,
}

impl Store {
pub fn open(path: &Path) -> Result<Self> {
    // Register sqlite-vec BEFORE opening the connection
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    let store = Self { conn };
    store.init_schema()?;
    Ok(store)
}

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch("
            -- Main thoughts table
            CREATE TABLE IF NOT EXISTS thoughts (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                content   TEXT NOT NULL,
                tags      TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- FTS5 virtual table for keyword search
            CREATE VIRTUAL TABLE IF NOT EXISTS thoughts_fts
                USING fts5(content, tags, content='thoughts', content_rowid='id');

            -- Trigger to keep FTS in sync on insert
            CREATE TRIGGER IF NOT EXISTS thoughts_ai AFTER INSERT ON thoughts BEGIN
                INSERT INTO thoughts_fts(rowid, content, tags)
                VALUES (new.id, new.content, new.tags);
            END;

            -- Vec0 virtual table for vector search
            -- 384 is the dimension of the MiniLM model fastembed uses
            CREATE VIRTUAL TABLE IF NOT EXISTS thoughts_vec
                USING vec0(embedding float[384]);
        ")?;

        Ok(())
    }

pub fn save_thought(
    &self,
    content: &str,
    tags: Option<&str>,
    embedding: &[f32],
) -> Result<i64> {
    // Insert the thought text and tags
    self.conn.execute(
        "INSERT INTO thoughts (content, tags) VALUES (?1, ?2)",
        rusqlite::params![content, tags],
    )?;

    let id = self.conn.last_insert_rowid();

    // Store the embedding in the vec0 table
    // sqlite-vec expects raw bytes (little-endian f32s)
    let bytes = embedding.as_bytes();
    self.conn.execute(
        "INSERT INTO thoughts_vec (rowid, embedding) VALUES (?1, ?2)",
        rusqlite::params![id, bytes],
    )?;

    Ok(id)
}

  pub fn list_thoughts(&self, limit: usize) -> Result<Vec<Thought>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, content, tags, created_at
         FROM thoughts
         ORDER BY created_at DESC
         LIMIT ?1"
    )?;

    let thoughts = stmt.query_map([limit], |row| {
        Ok(Thought {
            id: row.get(0)?,
            content: row.get(1)?,
            tags: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(thoughts)
}

}
