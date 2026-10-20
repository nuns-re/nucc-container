use binrw::{binrw, until_eof};

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkDynamics {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    #[bw(assert(spring_groups.len() <= u16::MAX as usize))]
    #[bw(calc = spring_groups.len() as u16)]
    pub spring_group_count: u16,
    #[bw(assert(collision_spheres.len() <= u16::MAX as usize))]
    #[bw(calc = collision_spheres.len() as u16)]
    pub collision_sphere_count: u16,
    #[brw(if(version >= 0x68))]
    pub clump_index: Option<u32>,

    #[br(count = spring_group_count)]
    #[br(args { inner: (version,) })]
    #[bw(args(version))]
    #[br(assert(spring_groups.iter().map(|g: &DynamicsSpringGroup| g.bone_count as usize).sum::<usize>() <= 1_048_576))]
    pub spring_groups: Vec<DynamicsSpringGroup>,
    #[br(count = collision_sphere_count)]
    #[br(args { inner: (version,) })]
    #[bw(args(version))]
    pub collision_spheres: Vec<DynamicsCollisionSphere>,
    #[br(count = spring_groups.iter().map(|g| g.bone_count as usize).sum::<usize>())]
    #[bw(assert(bone_flags.len() == spring_groups.iter().map(|g| g.bone_count as usize).sum::<usize>()))]
    pub bone_flags: Vec<u16>,
    #[br(parse_with = until_eof)]
    pub trailing_data: Vec<u8>,
}

impl NuccChunk for NuccChunkDynamics {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkDynamics
    }

    fn version(&self) -> u16 {
        self.version
    }
}

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DynamicsSpringGroup {
    pub bounciness: f32,
    pub elasticity: f32,
    pub stiffness: f32,
    #[br(if(version >= 0x67, 1.0))]
    #[bw(if(version >= 0x67))]
    pub movement: f32,
    pub coord_index: u16,
    pub bone_count: u16,
}

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DynamicsCollisionSphere {
    pub offset: [f32; 3],
    pub scale: [f32; 3],
    pub coord_index: u16,
    pub attach_groups: u16,
    #[br(if(version >= 0x69, -1))]
    #[bw(if(version >= 0x69))]
    pub spring_group_index: i16,
    #[brw(if(version >= 0x69))]
    pub unknown1e: u16,
    #[br(if(version >= 0x71 && attach_groups & 1 != 0))]
    #[bw(assert(attached_groups.len() <= u16::MAX as usize))]
    #[bw(if(version >= 0x71 && *attach_groups & 1 != 0), calc = if version >= 0x71 && *attach_groups & 1 != 0 { Some(attached_groups.len() as u16) } else { None })]
    pub attached_group_count: Option<u16>,
    #[br(count = attached_group_count.unwrap_or(0))]
    #[bw(assert((version >= 0x71 && *attach_groups & 1 != 0) || attached_groups.is_empty()))]
    pub attached_groups: Vec<u16>,
}
