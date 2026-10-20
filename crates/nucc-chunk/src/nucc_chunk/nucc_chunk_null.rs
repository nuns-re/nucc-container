use super::{NuccChunk, NuccChunkType};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkNull(pub u16);

impl NuccChunk for NuccChunkNull {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkNull
    }

    fn version(&self) -> u16 {
        self.0
    }
}
