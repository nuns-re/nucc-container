use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(import(version: u16))]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkMorphModel {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    #[brw(pad_after = 2)]
    pub count: u16,

    #[br(count = (count * 8) + 8)]
    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkMorphModel {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkMorphModel
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn is_opaque(&self) -> bool {
        true
    }
}
