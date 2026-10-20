use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big)]
#[brw(import(version: u16))]
#[derive(Debug, Clone, Default)]
pub struct NuccChunkParticle {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub settings_hdr: SectionHeader,  // ParticleHeaders[0]
    pub materials_hdr: SectionHeader, // ParticleHeaders[1]
    pub bones_hdr: SectionHeader,     // ParticleHeaders[2]
    pub force_hdr: SectionHeader,     // ParticleHeaders[3]
    pub timelines_hdr: SectionHeader, // ParticleHeaders[4]

    // Sequential sections (typed only)
    #[br(count = settings_hdr.entry_count)]
    pub particle_settings: Vec<ParticleSettingsV79>,

    #[br(count = materials_hdr.entry_count)]
    pub materials: Vec<ParticleMaterialEntry>,

    #[br(count = bones_hdr.entry_count)]
    pub bones: Vec<ParticleBoneEntry>,

    #[br(count = force_hdr.entry_count)]
    pub forcefields: Vec<ForcefieldEntry>,

    #[br(count = timelines_hdr.entry_count)]
    pub timelines: Vec<Timeline>,
}

impl NuccChunk for NuccChunkParticle {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkParticle
    }
    fn version(&self) -> u16 {
        self.version
    }
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SectionHeader {
    pub position: u32, // unused
    pub entry_count: u16,
    pub entry_size: u16,
}

/* ================== Section structures ================== */

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct ParticleSettingsV79 {
    pub chunkmap_id_animation: u32,
    pub entry_index: u32,

    #[br(count = 0x08)]
    #[bw(calc = vec![0u8; 8])]
    pub _skip0: Vec<u8>,

    pub stretch_loop: u8,
    pub attachment_type: u8,
    pub blowout_disabler: u8,
    pub attach_to_bone: u8,
    pub random_roll: u8,
    pub frame_rate: u8,
    pub unk1: u8,
    pub unk2: u8,

    #[br(count = 0x02)]
    #[bw(calc = vec![0u8; 2])]
    pub _pad0: Vec<u8>,

    pub unk_flag: u8,
    pub start_time: u8,
    pub end_time: i16,
    pub unk_after_end: u16,

    pub particle_count: f32,
    pub spawn_radius_start: f32,
    pub spawn_radius_end: f32,
    pub fade_time_u16: u16,

    #[br(count = 0x02)]
    #[bw(calc = vec![0u8; 2])]
    pub _pad1: Vec<u8>,

    pub fade_time2: f32,
    pub blowout: f32,

    pub rot_w: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,

    pub unk3: f32,
    pub transparency: f32,
    pub unk4: f32,
    pub unk5: f32,
    pub unk6: f32,

    pub anim_scale_0_x: f32,
    pub anim_scale_0_y: f32,
    pub anim_scale_0_z: f32,
    pub anim_scale_1_x: f32,
    pub anim_scale_1_y: f32,
    pub anim_scale_1_z: f32,
    pub anim_scale_2_x: f32,
    pub anim_scale_2_y: f32,
    pub anim_scale_2_z: f32,
    pub default_scale_x: f32,
    pub default_scale_y: f32,
    pub default_scale_z: f32,

    pub blend_scale_value: f32,

    pub col0_r: f32,
    pub col0_g: f32,
    pub col0_b: f32,
    pub col0_a: f32,
    pub col1_r: f32,
    pub col1_g: f32,
    pub col1_b: f32,
    pub col1_a: f32,
    pub col2_r: f32,
    pub col2_g: f32,
    pub col2_b: f32,
    pub col2_a: f32,

    pub blend_color_value: f32,

    #[br(count = 0x0C)]
    #[bw(calc = vec![0u8; 12])]
    pub _pad_tail: Vec<u8>,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct ParticleMaterialEntry {
    pub chunkmap_id_effect: u32,
    pub particle_entry_index: u32,

    #[brw(pad_before = 8)]
    pub unk1: i16,
    pub unk2: i16,

    #[brw(pad_before = 4)]
    pub unk3: i16,
    pub unk4: i16,
    pub chunkmap_id_type: u32, // 1=Clump, 2=Anm, 5=Billboard
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct ParticleBoneEntry {
    pub chunkmap_id_coord: u32,
    pub particle_entry_index: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: f32,
    pub unk5: f32,
    pub unk6: u32,
    pub unk7: u32,
    pub unk8: u32,
    pub unk9: u32,
    pub unk10: u32,
    pub chunkmap_id_clump: u32,
    pub unk12: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct ForcefieldEntry {
    #[br(count = 0x70)]
    pub data: Vec<u8>, // Placeholder; not yet mapped
}

/* ================== Timelines ================== */

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct Timeline {
    pub frame_count: u32,
    #[br(count = frame_count)]
    pub frames: Vec<u32>,
    #[br(if((frame_count * 4 + 4) % 8 != 0))]
    #[br(count = 4)]
    pub pad_to_8: Option<Vec<u8>>,
}
