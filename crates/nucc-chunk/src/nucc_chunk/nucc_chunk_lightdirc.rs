use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkLightDirc {
    #[br(calc = version)]
    #[bw(ignore)]
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

    fn is_opaque(&self) -> bool {
        true
    }
}
