use crate::{db::Store, embeddings::Embedder};
use anyhow::Result;
use std::io::{self, Read};

pub fn run(
    store: &Store,
    embedder: &Embedder,
    text: Option<&str>,
    tags: Option<&str>,
) -> Result<()> {
    let content = match text {
        Some(t) => t.to_string(),
        None => {
            eprintln!("Write your thought, then press ^D to save:");
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
    };

    if content.is_empty() {
        anyhow::bail!("Cannot save an empty thought.");
    }

    let embedding = embedder.embed(&content)?;
    let id = store.save_thought(&content, tags, &embedding)?;
    println!("✓ Saved thought #{}", id);
    Ok(())
}
