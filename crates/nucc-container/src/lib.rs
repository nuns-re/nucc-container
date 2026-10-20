//! High-level document layer for NUCC XFBIN files.

mod document;
#[cfg(feature = "nut")]
mod texture;

pub use document::{NuccChunkEntry, NuccXfbinDocument, NuccXfbinPage};
#[cfg(feature = "nut")]
pub use texture::NuccChunkTextureExt;
