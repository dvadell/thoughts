use crate::db::Store;
use anyhow::Result;

pub fn run(store: &Store, limit: usize) -> Result<()> {
    let thoughts = store.list_thoughts(limit)?;

    if thoughts.is_empty() {
        println!("No thoughts yet. Try: thoughts save \"your first thought\"");
        return Ok(());
    }

    for t in thoughts {
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
    }

    Ok(())
}
