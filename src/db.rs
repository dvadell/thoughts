use anyhow::Result;
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::Connection;
use sqlite_vec::sqlite3_vec_init;
use std::path::Path;
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
        self.conn.execute_batch(
            "
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
        ",
        )?;

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
         ORDER BY id DESC
         LIMIT ?1",
        )?;

        let thoughts = stmt
            .query_map([limit], |row| {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_store() -> Store {
        // Use :memory: for tests — fast, isolated, no files left behind
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        let store = Store { conn };
        store.init_schema().unwrap();
        store
    }

    #[test]
    fn test_save_and_list() {
        let store = in_memory_store();
        let embedding = vec![0.0f32; 384];

        store.save_thought("hello world", None, &embedding).unwrap();
        let thoughts = store.list_thoughts(10).unwrap();

        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].content, "hello world");
    }

    #[test]
    fn test_tags_are_saved() {
        let store = in_memory_store();
        let embedding = vec![0.0f32; 384];

        store
            .save_thought("tagged thought", Some("rust,cli"), &embedding)
            .unwrap();
        let thoughts = store.list_thoughts(10).unwrap();

        assert_eq!(thoughts[0].tags.as_deref(), Some("rust,cli"));
    }

    #[test]
    fn test_list_is_ordered_by_recency() {
        let store = in_memory_store();
        let embedding = vec![0.0f32; 384];

        store.save_thought("first", None, &embedding).unwrap();
        store.save_thought("second", None, &embedding).unwrap();

        let thoughts = store.list_thoughts(10).unwrap();
        assert_eq!(thoughts[0].content, "second");
        assert_eq!(thoughts[1].content, "first");
    }
}
