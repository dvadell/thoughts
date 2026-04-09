use crate::{db::Store, embeddings::Embedder};
use anyhow::Result;

pub fn run(store: &Store, embedder: &Embedder, query: &str, limit: usize) -> Result<()> {
    let embedding = embedder.embed(query)?;
    let results = store.search_hybrid(&embedding, query, limit)?;

    if results.is_empty() {
        println!("No thoughts found.");
        return Ok(());
    }

    for result in results {
        let t = &result.thought;
        println!(
            "\n[#{}] {} {}",
            t.id,
            t.created_at,
            t.tags
                .as_deref()
                .map(|tg| format!("[{}]", tg))
                .unwrap_or_default()
        );
        println!("  {}", t.content);
        println!("  score: {:.4}", result.score);
    }

    Ok(())
}
