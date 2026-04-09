use crate::{db::Store, embeddings::Embedder};
use anyhow::Result;

pub fn run(store: &Store, embedder: &Embedder, text: &str, tags: Option<&str>) -> Result<()> {
    let embedding = embedder.embed(text)?;
    let id = store.save_thought(text, tags, &embedding)?;
    println!("✓ Saved thought #{}", id);
    Ok(())
}
