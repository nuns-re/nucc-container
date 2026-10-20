use binrw::{binrw, until_eof};

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkCoord {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    #[br(parse_with = until_eof)]
    pub trailing_data: Vec<u8>,
}

impl NuccChunk for NuccChunkCoord {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkCoord
    }

    fn version(&self) -> u16 {
        self.version
    }
}
