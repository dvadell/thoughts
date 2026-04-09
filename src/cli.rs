use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "thoughts", about = "A personal thought logger with semantic search")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Save a new thought
    Save {
        /// The thought text
        text: String,

        /// Optional tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Search thoughts semantically
    Search {
        /// Your search query
        query: String,

        /// Max number of results
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },

    /// List recent thoughts
    List {
        /// Max number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}
