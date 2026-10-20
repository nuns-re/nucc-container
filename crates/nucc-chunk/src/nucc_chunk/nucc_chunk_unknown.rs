use super::{NuccChunk, NuccChunkType};

#[derive(Debug, Clone, PartialEq, Default)]

pub struct NuccChunkUnknown {
    pub version: u16,

    pub chunk_type: String,

    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkUnknown {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkUnknown
    }

    fn version(&self) -> u16 {
        self.version
    }
}
