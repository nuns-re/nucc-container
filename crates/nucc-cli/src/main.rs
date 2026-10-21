use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use nucc_container::{Acb, NuccChunkNub, NuccXfbinDocument};

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
    /// List cues and waveforms in ACB chunks or a raw ACB file.
    Acb {
        #[arg(value_name = "XFBIN_OR_ACB")]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { path } => inspect(&path),
        Command::Chunks { path } => chunks(&path),
        Command::Acb { path } => acb(&path),
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

fn acb(path: &PathBuf) -> anyhow::Result<()> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    if bytes.starts_with(b"@UTF") {
        let acb = Acb::parse(&bytes)
            .with_context(|| format!("failed to parse ACB {}", path.display()))?;
        print_acb(&acb);
        return Ok(());
    }

    let document = NuccXfbinDocument::parse(&bytes)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    let mut found = false;

    for entry in &document.chunks {
        let Some(nub) = entry.chunk.downcast_ref::<NuccChunkNub>() else {
            continue;
        };
        let acb = nub
            .acb()
            .with_context(|| format!("failed to parse ACB chunk {}", entry.chunk_name()))?;
        found = true;
        print_acb(&acb);
    }

    if !found {
        println!("no nuccChunkNub chunks");
    }
    Ok(())
}

fn print_acb(acb: &Acb) {
    println!(
        "{}  cues={}  waveforms={}  awb={}",
        acb.header.name, acb.header.cue_count, acb.header.waveform_count, acb.header.awb_size
    );
    for cue in &acb.cue_names {
        let indices = acb
            .cue_waveforms
            .get(cue.cue_index as usize)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let waves: Vec<_> = indices
            .iter()
            .filter_map(|index| acb.waveforms.get(*index))
            .map(|wave| format!("{}:{}", wave.id, wave.stream_awb_id.unwrap_or(u16::MAX)))
            .collect();
        println!(
            "  {:04}  {:<48}  {}",
            cue.cue_index,
            cue.name,
            waves.join(", ")
        );
    }
}
