use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

use super::nucc_chunk_anm::AnmCoord;

#[binrw]
#[brw(big)]
#[brw(import(_version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkAnmStrmFrame {
    #[brw(ignore)]
    pub version: u16,

    pub frame_number: u32,

    #[bw(calc = entries.len() as u16)]
    pub entry_count: u16,

    pub unknown: u16,

    #[br(count = entry_count)]
    pub entries: Vec<AnmStrmEntry>,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmStrmEntry {
    pub coord: AnmCoord,
    pub entry_format: u16,
    pub entry_size: u16,

    #[br(args(entry_format))]
    pub entry_data: Entry,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
#[br(import(entry_format: u16))]
pub enum Entry {
    #[br(pre_assert(entry_format == 1))]
    Bone(AnmEntryCoord),

    #[br(pre_assert(entry_format == 2))]
    Camera(AnmEntryCamera),

    #[br(pre_assert(entry_format == 4))]
    Material(AnmEntryMaterial),

    #[br(pre_assert(entry_format == 5))]
    LightDirc(AnmEntryLightDirc),

    #[br(pre_assert(entry_format == 6))]
    LightPoint(AnmEntryLightPoint),

    #[br(pre_assert(entry_format == 8))]
    Ambient(AnmEntryAmbient),

    #[br(pre_assert(entry_format == 12))]
    MorphModel(AnmEntryMorphModel),

    #[br(pre_assert(false))]
    Unknown,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmEntryCoord {
    pub frame_count: i32,
    pub location: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub scale: (f32, f32, f32),
    pub toggled: f32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmEntryCamera {
    pub frame_count: i32,
    pub location: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub fov: f32,
    pub scale: (f32, f32, f32),
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmEntryMaterial {
    pub frame_count: i32,
    pub ambient_color: [f32; 16],
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmEntryLightDirc {
    pub frame_count: i32,
    pub color: (f32, f32, f32),
    pub intensity: f32,
    pub direction: (f32, f32, f32, f32),
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmEntryLightPoint {
    pub frame_count: i32,
    pub color: (f32, f32, f32),
    pub position: (f32, f32, f32),
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmEntryAmbient {
    pub frame_count: i32,
    pub color: (f32, f32, f32),
    pub intensity: f32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmEntryMorphModel {
    pub frame_count: i32,
    #[br(count = frame_count)]
    pub morph_weight: Vec<f32>,
}

impl NuccChunk for NuccChunkAnmStrmFrame {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAnmStrmFrame
    }

    fn version(&self) -> u16 {
        self.version
    }
}
