use binrw::binrw;

use super::nucc_chunk_anm::CoordParent;
use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big)]
#[brw(import(version: u16))]
#[derive(Debug, Clone, PartialEq)]

pub struct NuccChunkAnmStrm {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub frame_count: u32,

    #[bw(calc = 100)]
    pub frame_size: u32,

    #[bw(calc = entries.len() as u16)]
    pub entry_count: u16,

    pub is_looped: u16,

    #[bw(calc = clumps.len() as u16)]
    pub clump_count: u16,

    #[bw(calc = other_entry_indices.len() as u16)]
    pub other_entry_count: u16,

    pub unk_entry_count: u16,

    #[bw(calc = coord_parents.len() as u16)]
    pub coord_count: u16,

    #[br(count = clump_count)]
    pub clumps: Vec<AnmStrmClump>,

    #[br(count = other_entry_count + unk_entry_count)]
    pub other_entry_indices: Vec<u32>,

    #[br(count = coord_count)]
    pub coord_parents: Vec<CoordParent>,

    #[br(count = entry_count)]
    pub entries: Vec<AnmStrmFrameInfo>,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmStrmClump {
    pub clump_index: u32,

    #[bw(calc = bone_material_indices.len() as u16)]
    pub bone_material_count: u16,

    #[bw(calc = model_indices.len() as u16)]
    pub model_count: u16,

    #[br(count = bone_material_count)]
    pub bone_material_indices: Vec<u32>,

    #[br(count = model_count)]
    #[br(pad_after = 4 * (model_count as usize))]
    pub model_indices: Vec<u32>,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmStrmFrameInfo {
    pub unknown: u16,
    pub frame_offset: u16,
    pub frame_number: u32,
}

impl NuccChunk for NuccChunkAnmStrm {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAnmStrm
    }

    fn version(&self) -> u16 {
        self.version
    }
}
