use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkLayerSet {
    #[br(calc = version)] // write imported version into the field when reading
    #[bw(ignore)] // do not write this field back out
    pub version: u16,

    #[br(count = 466)]
    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkLayerSet {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkLayerSet
    }

    fn version(&self) -> u16 {
        self.version
    }
}
