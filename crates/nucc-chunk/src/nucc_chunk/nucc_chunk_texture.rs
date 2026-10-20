use binrw::{binrw, until_eof};

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkTexture {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    pub field00: u16,
    pub width: u16,
    pub height: u16,
    pub field06: u16,
    #[bw(calc = nut_data.len() as u32)]
    #[br(assert(size <= 512 * 1024 * 1024, "NUT payload exceeds limit"))]
    pub size: u32,
    #[br(count = size)]
    pub nut_data: Vec<u8>,
    #[br(parse_with = until_eof)]
    pub trailing_data: Vec<u8>,
}

impl NuccChunk for NuccChunkTexture {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkTexture
    }

    fn version(&self) -> u16 {
        self.version
    }
}
