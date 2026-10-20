//! XFBIN container layer.

mod data;
mod parser;

pub use data::{
    NuccXfbinChunk, NuccXfbinChunkIndex, NuccXfbinChunkMap, NuccXfbinChunkReference, NuccXfbinData,
    NuccXfbinHeader,
};
pub use parser::{NuccXfbinChunkEntry, NuccXfbinParser};
