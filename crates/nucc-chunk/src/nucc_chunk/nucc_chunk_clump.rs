use binrw::{binrw, until_eof};

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkClump {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    pub flags: u32,
    #[bw(calc = parents.len() as u16)]
    pub coord_count: u16,
    pub unknown06: u16,
    #[br(if(flags == 2))]
    #[bw(if(*flags == 2))]
    pub extended_header: Option<[u8; 28]>,
    #[br(count = coord_count)]
    pub parents: Vec<i16>,
    #[br(count = coord_count)]
    pub coord_indices: Vec<u32>,
    #[br(parse_with = until_eof)]
    pub trailing_data: Vec<u8>,
}

impl NuccChunk for NuccChunkClump {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkClump
    }

    fn version(&self) -> u16 {
        self.version
    }
}
