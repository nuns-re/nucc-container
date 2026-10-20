//! Shared NUCC container types.

pub mod chunk_index;
pub mod chunk_type;
pub mod endian;
pub mod error;

pub use chunk_index::{NuccChunkExternalIndex, NuccChunkIndex, NuccChunkInternalIndex};
pub use chunk_type::NuccChunkType;
pub use endian::NuccEndianType;
pub use error::{NuccError, Result};
