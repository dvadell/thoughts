mod cli;
mod commands;
mod db;
mod embeddings;
mod search;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use db::Store;
use embeddings::Embedder;
use std::path::PathBuf;

fn data_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("thoughts");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("db.sqlite")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = Store::open(&data_path())?;

    match cli.command {
        Command::Save { text, tags } => {
            let embedder = Embedder::new()?;
            commands::save::run(&store, &embedder, &text, tags.as_deref())?;
        }
        Command::Search { query, limit } => {
            let embedder = Embedder::new()?;
            commands::search::run(&store, &embedder, &query, limit)?;
        }
        Command::List { limit } => {
            commands::list::run(&store, limit)?;
        }
    }

    Ok(())
}
