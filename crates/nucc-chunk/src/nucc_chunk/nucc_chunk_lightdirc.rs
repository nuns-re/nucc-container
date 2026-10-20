use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkLightDirc {
    #[br(calc = version)] // write imported version into the field when reading
    #[bw(ignore)] // do not write this field back out
    pub version: u16,

    #[br(count = 64)]
    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkLightDirc {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkLightDirc
    }

    fn version(&self) -> u16 {
        self.version
    }
}
