use binrw::{binrw, until_eof};

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkModel {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    pub unknown00: u32,
    pub attributes: u16,
    pub rigging_flag: u16,
    #[brw(if(version > 0x73 && !(0x74..0x76).contains(&version)))]
    pub extra_header: Option<u32>,
    pub clump_index: u32,
    pub hit_index: u32,
    pub coord_index: u32,
    #[bw(calc = nud_data.len() as u32)]
    #[br(assert(nud_size <= 512 * 1024 * 1024, "NUD payload exceeds limit"))]
    pub nud_size: u32,
    #[brw(if((0x74..0x76).contains(&version)))]
    pub extra_header_74: Option<u32>,
    #[brw(if(attributes & 4 != 0))]
    pub bounds: Option<[f32; 6]>,
    #[br(count = nud_size)]
    pub nud_data: Vec<u8>,
    #[bw(calc = material_indices.len() as u16)]
    pub material_count: u16,
    #[br(count = material_count)]
    pub material_indices: Vec<u32>,
    #[br(parse_with = until_eof)]
    pub trailing_data: Vec<u8>,
}

impl NuccChunk for NuccChunkModel {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkModel
    }

    fn version(&self) -> u16 {
        self.version
    }
}

#[cfg(feature = "nu")]
impl NuccChunkModel {
    pub fn nud(&self) -> nucc_core::Result<nu::model::Nud> {
        nu::model::Nud::parse(&self.nud_data)
            .map_err(|error| nucc_core::NuccError::InvalidChunk(format!("NUD: {error}")))
    }
}
