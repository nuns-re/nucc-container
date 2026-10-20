use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkAmbient {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    #[br(count = 16)]
    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkAmbient {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAmbient
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn is_opaque(&self) -> bool {
        true
    }
}
