//! Chunk index records used by `nuccXfbinDocument`.

use crate::NuccChunkType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NuccChunkInternalIndex {
    pub chunk_name: String,
    pub chunk_type: NuccChunkType,
    pub filepath: String,
    pub link: Option<usize>,
    pub original_name: Option<String>,
    pub original_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NuccChunkExternalIndex {
    pub chunk_name: String,
    pub chunk_type: NuccChunkType,
    pub filepath: String,
    pub link: Option<usize>,
    pub original_name: Option<String>,
    pub original_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NuccChunkIndex {
    Internal(NuccChunkInternalIndex),
    External(NuccChunkExternalIndex),
}

impl NuccChunkIndex {
    pub fn chunk_name(&self) -> &str {
        match self {
            Self::Internal(index) => &index.chunk_name,
            Self::External(index) => &index.chunk_name,
        }
    }

    pub fn chunk_type(&self) -> NuccChunkType {
        match self {
            Self::Internal(index) => index.chunk_type,
            Self::External(index) => index.chunk_type,
        }
    }

    pub fn filepath(&self) -> &str {
        match self {
            Self::Internal(index) => &index.filepath,
            Self::External(index) => &index.filepath,
        }
    }

    pub fn link(&self) -> Option<usize> {
        match self {
            Self::Internal(index) => index.link,
            Self::External(index) => index.link,
        }
    }

    pub fn original_name(&self) -> Option<&str> {
        match self {
            Self::Internal(index) => index.original_name.as_deref(),
            Self::External(index) => index.original_name.as_deref(),
        }
    }

    pub fn original_path(&self) -> Option<&str> {
        match self {
            Self::Internal(index) => index.original_path.as_deref(),
            Self::External(index) => index.original_path.as_deref(),
        }
    }
}
