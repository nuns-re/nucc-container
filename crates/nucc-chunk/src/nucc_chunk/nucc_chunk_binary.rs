use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkBinary {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    #[bw(calc = data.len() as u32)]
    #[brw(big)]
    pub size: u32,

    #[br(count = size)]
    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkBinary {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkBinary
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn extension(&self) -> String {
        String::from(".binary")
    }
}
