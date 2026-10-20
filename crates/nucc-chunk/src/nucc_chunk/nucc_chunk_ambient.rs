use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkAmbient {
    #[br(calc = version)] // write imported version into the field when reading
    #[bw(ignore)] // do not write this field back out
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
}
