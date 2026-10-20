use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use nucc_container::NuccXfbinDocument;

#[derive(Debug, Parser)]
#[command(name = "nucc", version, about = "Inspect NUCC XFBIN containers")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print a summary of an XFBIN file.
    Inspect {
        #[arg(value_name = "XFBIN")]
        path: PathBuf,
    },
    /// List the chunks in an XFBIN file.
    Chunks {
        #[arg(value_name = "XFBIN")]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { path } => inspect(&path),
        Command::Chunks { path } => chunks(&path),
    }
}

fn load(path: &PathBuf) -> anyhow::Result<NuccXfbinDocument> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    NuccXfbinDocument::parse(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn inspect(path: &PathBuf) -> anyhow::Result<()> {
    let document = load(path)?;
    println!("version: {}", document.version);
    println!("chunks: {}", document.chunks.len());
    println!("references: {}", document.references.len());
    println!("pages: {}", document.pages.len());
    Ok(())
}

fn chunks(path: &PathBuf) -> anyhow::Result<()> {
    let document = load(path)?;
    for (index, chunk) in document.chunks.iter().enumerate() {
        println!(
            "{index:04}  {:<32}  v{:<4}  {:<4}  {}",
            chunk.chunk_type(),
            chunk.chunk.version(),
            if chunk.is_typed() {
                "typed"
            } else if chunk.is_opaque() {
                "opaque"
            } else {
                "raw"
            },
            chunk.chunk_name()
        );
    }
    Ok(())
}
