//! High-level document layer for NUCC XFBIN files.

mod document;

pub use document::{NuccChunkEntry, NuccXfbinDocument, NuccXfbinPage};
pub use nucc_chunk::{NuccChunkModel, NuccChunkNub, NuccChunkTexture};
