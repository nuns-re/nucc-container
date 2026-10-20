use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big)]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkCamera {
    #[br(calc = version)] // write imported version into the field when reading
    #[bw(ignore)] // do not write this field back out
    pub version: u16,

    #[brw(pad_before = 4)]
    pub fov: f32,
}

impl NuccChunk for NuccChunkCamera {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkCamera
    }

    fn version(&self) -> u16 {
        self.version
    }
}
