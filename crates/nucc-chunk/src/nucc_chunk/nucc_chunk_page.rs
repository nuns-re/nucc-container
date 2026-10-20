use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big)]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkPage {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub map_index_count: u32,
    pub reference_count: u32,
}

impl NuccChunk for NuccChunkPage {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkPage
    }

    fn version(&self) -> u16 {
        self.version
    }
}
