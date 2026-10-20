//! Raw serialized XFBIN layouts.

use binrw::{binrw, until_eof, NullString};
use std::mem::size_of;

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Default)]
pub struct NuccXfbinData {
    pub header: NuccXfbinHeader,
    pub index: NuccXfbinChunkIndex,

    #[br(parse_with = until_eof)]
    pub chunks: Vec<NuccXfbinChunk>,
}

#[binrw]
#[brw(big, magic = b"NUCC")]
#[derive(Debug, Clone, Default)]
pub struct NuccXfbinHeader {
    pub version: u32,
    #[brw(pad_after = 6)]
    pub encrypted: u16,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Default)]
pub struct NuccXfbinChunkIndex {
    #[bw(calc = self.get_chunk_table_size())]
    pub chunk_table_size: u32,

    pub min_page_size: u32,

    #[brw(pad_after = 2)]
    pub version: u16,

    #[bw(calc = self.chunk_types.len() as u32)]
    pub chunk_type_count: u32,

    #[bw(calc = self.chunk_types.iter().map(|x| (x.len() + 1) as u32).sum())]
    pub chunk_type_size: u32,

    #[bw(calc = self.filepaths.len() as u32)]
    pub filepath_count: u32,

    #[bw(calc = self.filepaths.iter().map(|x| (x.len() + 1) as u32).sum())]
    pub filepath_size: u32,

    #[bw(calc = self.chunk_names.len() as u32)]
    pub chunk_name_count: u32,

    #[bw(calc = self.chunk_names.iter().map(|x| (x.len() + 1) as u32).sum())]
    pub chunk_name_size: u32,

    #[bw(calc = self.chunk_maps.len() as u32)]
    pub chunk_map_count: u32,

    #[bw(calc = (self.chunk_maps.len() * size_of::<NuccXfbinChunkMap>()) as u32)]
    pub chunk_map_size: u32,

    #[bw(calc = self.chunk_map_indices.len() as u32)]
    pub chunk_map_indices_count: u32,

    #[bw(calc = self.chunk_references.len() as u32)]
    pub references_count: u32,

    #[br(count = chunk_type_count)]
    pub chunk_types: Vec<NullString>,

    #[br(count = filepath_count)]
    pub filepaths: Vec<NullString>,

    #[br(count = chunk_name_count)]
    pub chunk_names: Vec<NullString>,

    #[brw(align_before = 4)]
    #[br(count = chunk_map_count)]
    pub chunk_maps: Vec<NuccXfbinChunkMap>,

    #[br(count = references_count)]
    pub chunk_references: Vec<NuccXfbinChunkReference>,

    #[br(count = chunk_map_indices_count)]
    pub chunk_map_indices: Vec<u32>,
}

impl NuccXfbinChunkIndex {
    pub fn get_chunk_table_size(&self) -> u32 {
        let chunk_types_size: u32 = self.chunk_types.iter().map(|x| (x.len() + 1) as u32).sum();
        let filepaths_size: u32 = self.filepaths.iter().map(|x| (x.len() + 1) as u32).sum();
        let chunk_names_size: u32 = self.chunk_names.iter().map(|x| (x.len() + 1) as u32).sum();
        let string_sizes = chunk_types_size + filepaths_size + chunk_names_size;
        let alignment = (4 - (string_sizes % 4)) % 4;

        0x28 + string_sizes
            + alignment
            + (self.chunk_maps.len() as u32 * size_of::<NuccXfbinChunkMap>() as u32)
            + (self.chunk_map_indices.len() as u32 * size_of::<u32>() as u32)
    }
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NuccXfbinChunkReference {
    pub chunk_name_index: u32,
    pub chunk_map_index: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NuccXfbinChunkMap {
    pub chunk_type_index: u32,
    pub filepath_index: u32,
    pub chunk_name_index: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccXfbinChunk {
    #[bw(calc = data.len() as u32)]
    pub size: u32,

    pub chunk_map_index: u32,

    #[brw(pad_after = 2)]
    pub version: u16,

    #[br(count = size)]
    pub data: Vec<u8>,
}
