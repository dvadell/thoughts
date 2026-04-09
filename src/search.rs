use crate::db::{Store, Thought};
use anyhow::Result;
use zerocopy::AsBytes;

pub struct SearchResult {
    pub thought: Thought,
    pub score: f64,        // combined RRF score
}

impl Store {
    pub fn search_vector(&self, embedding: &[f32], limit: usize) -> Result<Vec<(i64, f64)>> {
        let bytes = embedding.as_bytes();

        let mut stmt = self.conn.prepare(
            "SELECT rowid, distance
             FROM thoughts_vec
             WHERE embedding MATCH ?1
             ORDER BY distance
             LIMIT ?2"
        )?;

        let results = stmt.query_map(
            rusqlite::params![bytes, limit * 2],  // fetch extra for fusion
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(results)
    }

    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<(i64, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT rowid, rank
             FROM thoughts_fts
             WHERE thoughts_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;

        let results = stmt.query_map(
            rusqlite::params![query, limit * 2],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(results)
    }

    pub fn search_hybrid(&self, embedding: &[f32], query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let vec_results = self.search_vector(embedding, limit)?;
        let fts_results = self.search_fts(query, limit)?;

        // Reciprocal Rank Fusion: score = 1/(k + rank) summed across both lists
        let k = 60.0_f64;
        let mut scores: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();

        for (rank, (id, _)) in vec_results.iter().enumerate() {
            *scores.entry(*id).or_insert(0.0) += 1.0 / (k + rank as f64 + 1.0);
        }
        for (rank, (id, _)) in fts_results.iter().enumerate() {
            *scores.entry(*id).or_insert(0.0) += 1.0 / (k + rank as f64 + 1.0);
        }

        // Sort by combined score, fetch top results
        let mut ranked: Vec<(i64, f64)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked.truncate(limit);

        // Fetch full thought data for top IDs
        let mut results = Vec::new();
        for (id, score) in ranked {
            let thought = self.get_thought_by_id(id)?;
            results.push(SearchResult { thought, score });
        }

        Ok(results)
    }

    pub fn get_thought_by_id(&self, id: i64) -> Result<Thought> {
        Ok(self.conn.query_row(
            "SELECT id, content, tags, created_at FROM thoughts WHERE id = ?1",
            [id],
            |row| Ok(Thought {
                id: row.get(0)?,
                content: row.get(1)?,
                tags: row.get(2)?,
                created_at: row.get(3)?,
            }),
        )?)
    }
}
