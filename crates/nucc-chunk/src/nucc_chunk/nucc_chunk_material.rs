use binrw::{binrw, until_eof};

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkMaterial {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    #[bw(calc = texture_groups.len() as u16)]
    pub group_count: u16,
    pub unknown02: [u8; 6],
    pub flags: u32,
    #[brw(if(flags & 1 != 0))]
    pub uv0: Option<[f32; 4]>,
    #[brw(if(flags & 2 != 0))]
    pub uv1: Option<[f32; 4]>,
    #[brw(if(flags & 4 != 0))]
    pub uv2: Option<[f32; 4]>,
    #[brw(if(flags & 8 != 0))]
    pub uv3: Option<[f32; 4]>,
    #[brw(if(flags & 16 != 0))]
    pub field10: Option<[u8; 8]>,
    #[brw(if(flags & 32 != 0))]
    pub field20: Option<u32>,
    #[brw(if(flags & 64 != 0))]
    pub field40: Option<u32>,
    #[brw(if(flags & 128 != 0))]
    pub field80: Option<u32>,
    #[br(count = group_count)]
    pub texture_groups: Vec<MaterialTextureGroup>,
    #[br(parse_with = until_eof)]
    pub trailing_data: Vec<u8>,
}

impl NuccChunk for NuccChunkMaterial {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkMaterial
    }

    fn version(&self) -> u16 {
        self.version
    }
}

impl NuccChunkMaterial {
    pub fn shader_channels(&self) -> [f32; 3] {
        let rates = self.field10.unwrap_or([0; 8]);
        [
            f32::from_be_bytes(rates[..4].try_into().unwrap()),
            f32::from_be_bytes(rates[4..].try_into().unwrap()),
            f32::from_bits(self.field20.unwrap_or(0)),
        ]
    }

    pub fn uv_transforms(&self) -> [[f32; 4]; 4] {
        [self.uv0, self.uv1, self.uv2, self.uv3].map(|uv| uv.unwrap_or([0., 0., 1., 1.]))
    }

    pub fn shader_common(&self) -> [f32; 2] {
        [
            self.unknown02[0] as f32 / 255.,
            f32::from_be_bytes(self.unknown02[2..6].try_into().unwrap()),
        ]
    }
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialTextureGroup {
    #[bw(calc = texture_indices.len() as u16)]
    pub texture_count: u16,
    pub unknown02: [u8; 6],
    #[br(count = texture_count)]
    pub texture_indices: Vec<u32>,
}
