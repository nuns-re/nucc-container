use std::io::Write;

use binrw::{binrw, io::Cursor, BinRead, BinWrite};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkCoord, NuccChunkReference, NuccChunkType};

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

    #[br(parse_with = binrw::until_eof)]
    pub trailing_data: Vec<u8>,

    #[br(calc = None)]
    #[bw(ignore)]
    pub reference: Option<NuccChunkReference>,
    #[br(calc = None)]
    #[bw(ignore)]
    pub coord: Option<NuccChunkCoord>,
    #[br(calc = None)]
    #[bw(ignore)]
    pub raw_data: Option<Vec<u8>>,
}

impl NuccChunk for NuccChunkParticle {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkParticle
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if input.len() == 8 {
            let reference = NuccChunkReference {
                indices: [read_u32be(input, 0)?, read_u32be(input, 4)?],
            };
            let particle = NuccChunkParticle {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(particle)));
        }

        if input.len() == 42 {
            if let Ok((_, chunk)) = NuccChunkCoord::read_boxed(input, version) {
                if let Ok(coord) = chunk.downcast::<NuccChunkCoord>() {
                    let particle = NuccChunkParticle {
                        version,
                        coord: Some(*coord),
                        ..Default::default()
                    };
                    return Ok((input.to_vec(), Box::new(particle)));
                }
            }
        }

        match decode_particle(input, version) {
            Ok(particle) => Ok((input.to_vec(), Box::new(particle))),
            Err(_) => {
                let particle = NuccChunkParticle {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(particle)))
            }
        }
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let particle = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = particle.reference {
            for index in reference.indices {
                output.write_all(&index.to_be_bytes())?;
            }
            return Ok(());
        }
        if let Some(raw) = &particle.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        if let Some(coord) = &particle.coord {
            NuccChunkCoord::write_boxed(Box::new(coord.clone()), output, version)?;
            return Ok(());
        }
        write_particle(&particle, version, output)
    }
}

fn write_particle(
    particle: &NuccChunkParticle,
    version: u16,
    output: &mut Cursor<Vec<u8>>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if !matches!(version, 0x73 | 0x74 | 0x75 | 0x77 | 0x79 | 0x7a | 0x7b) {
        return Err(Box::new(NuccError::Unsupported(format!(
            "particle version {version:#x}"
        ))));
    }

    particle.settings_hdr.write_be(output)?;
    particle.materials_hdr.write_be(output)?;
    particle.bones_hdr.write_be(output)?;
    particle.force_hdr.write_be(output)?;
    particle.timelines_hdr.write_be(output)?;

    for setting in &particle.particle_settings {
        setting.write_be(output)?;
    }
    for material in &particle.materials {
        material.write_be(output)?;
    }

    let bone_size = if version < 0x79 { 48 } else { 56 };
    for bone in &particle.bones {
        output.write_all(&prefix(bone, bone_size)?)?;
    }

    let force_size = if version == 0x77 { 96 } else { 112 };
    for field in &particle.forcefields {
        output.write_all(&prefix(field, force_size)?)?;
    }

    for timeline in &particle.timelines {
        timeline.write_be(output)?;
    }
    output.write_all(&particle.trailing_data)?;
    Ok(())
}

fn prefix<T>(value: &T, length: usize) -> Result<Vec<u8>>
where
    T: BinWrite,
    T::Args: Default,
{
    let mut cursor = Cursor::new(Vec::new());
    value
        .write_be(&mut cursor)
        .map_err(|error| NuccError::InvalidChunk(format!("particle row: {error}")))?;
    let mut bytes = cursor.into_inner();
    if bytes.len() < length {
        return Err(NuccError::InvalidChunk("particle row is truncated".into()));
    }
    bytes.truncate(length);
    Ok(bytes)
}

fn decode_particle(input: &[u8], version: u16) -> Result<NuccChunkParticle> {
    if !matches!(version, 0x73 | 0x74 | 0x75 | 0x77 | 0x79 | 0x7a | 0x7b) {
        return Err(NuccError::Unsupported(format!(
            "particle version {version:#x}"
        )));
    }
    if input.len() < 40 {
        return Err(NuccError::Truncated);
    }

    let mut counts = [0usize; 5];
    let mut sizes = [0usize; 5];
    for index in 0..5 {
        let at = index * 8;
        counts[index] = u16::from_be_bytes([input[at + 4], input[at + 5]]) as usize;
        sizes[index] = u16::from_be_bytes([input[at + 6], input[at + 7]]) as usize;
    }

    let mut offset = 40usize;
    for (index, base_stride) in [208usize, 32, 56, 112].into_iter().enumerate() {
        let stride = if index == 2 && version < 0x79 {
            48
        } else if index == 3 && version == 0x77 {
            96
        } else {
            base_stride
        };
        let expected = counts[index]
            .checked_mul(stride)
            .ok_or(NuccError::Overflow)?;
        if sizes[index] != expected {
            return Err(NuccError::InvalidChunk(format!(
                "particle section {index} has size {:#x}, expected {expected:#x}",
                sizes[index]
            )));
        }
        offset = offset
            .checked_add(sizes[index])
            .ok_or(NuccError::Overflow)?;
        if offset > input.len() {
            return Err(NuccError::Truncated);
        }
    }

    let timeline_start = offset;
    for _ in 0..counts[4] {
        let bytes = input.get(offset..offset + 4).ok_or(NuccError::Truncated)?;
        let frame_count = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let available = input
            .len()
            .saturating_sub(offset + 4)
            .checked_div(4)
            .ok_or(NuccError::Overflow)?;
        if frame_count > available {
            return Err(NuccError::InvalidChunk(
                "particle timeline exceeds payload".into(),
            ));
        }
        let consumed = 4usize
            .checked_add(frame_count.checked_mul(4).ok_or(NuccError::Overflow)?)
            .ok_or(NuccError::Overflow)?;
        offset = offset
            .checked_add((consumed + 7) & !7)
            .ok_or(NuccError::Overflow)?;
        if offset > input.len() {
            return Err(NuccError::Truncated);
        }
    }
    if offset - timeline_start != sizes[4] {
        return Err(NuccError::InvalidChunk(
            "particle timeline size mismatch".into(),
        ));
    }

    if version == 0x7b && offset < input.len() {
        let bytes = input.get(offset..offset + 8).ok_or(NuccError::Truncated)?;
        let cache_size = usize::try_from(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
        .map_err(|_| NuccError::Overflow)?;
        if input.len() != offset + 8 + cache_size {
            return Err(NuccError::InvalidChunk(
                "particle cache size mismatch".into(),
            ));
        }
        offset = input.len();
    } else if offset != input.len() {
        return Err(NuccError::InvalidChunk("particle trailing data".into()));
    }

    let s0 = 40usize;
    let s1 = s0.checked_add(sizes[0]).ok_or(NuccError::Overflow)?;
    let s2 = s1.checked_add(sizes[1]).ok_or(NuccError::Overflow)?;
    let s3 = s2.checked_add(sizes[2]).ok_or(NuccError::Overflow)?;
    let s4 = s3.checked_add(sizes[3]).ok_or(NuccError::Overflow)?;
    let legacy_bones = version < 0x79 && counts[2] > 0;
    let nested_force_fields = version == 0x77 && counts[3] > 0;

    let normalized = if legacy_bones || nested_force_fields {
        let capacity = input
            .len()
            .checked_add(counts[2].checked_mul(8).ok_or(NuccError::Overflow)?)
            .and_then(|value| value.checked_add(counts[3].checked_mul(16)?))
            .ok_or(NuccError::Overflow)?;
        let mut expanded = Vec::with_capacity(capacity);
        expanded.extend_from_slice(&input[..s2]);
        if legacy_bones {
            for row in input[s2..s3].as_chunks::<48>().0 {
                expanded.extend_from_slice(row);
                expanded.extend_from_slice(&[0u8; 8]);
            }
        } else {
            expanded.extend_from_slice(&input[s2..s3]);
        }
        if nested_force_fields {
            for row in input[s3..s4].as_chunks::<96>().0 {
                expanded.extend_from_slice(row);
                expanded.extend_from_slice(&[0u8; 16]);
            }
        } else {
            expanded.extend_from_slice(&input[s3..s4]);
        }
        expanded.extend_from_slice(&input[s4..offset]);
        expanded
    } else {
        input[..offset].to_vec()
    };

    let mut cursor = Cursor::new(normalized);
    NuccChunkParticle::read_be_args(&mut cursor, (version,))
        .map_err(|error| NuccError::InvalidChunk(format!("particle: {error}")))
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

    pub force_field_flags_hi: u16,

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

fn read_u32be(data: &[u8], at: usize) -> Result<u32> {
    let bytes = data.get(at..at + 4).ok_or(NuccError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_all_particle_versions() {
        for version in [0x73, 0x74, 0x75, 0x77, 0x79, 0x7a, 0x7b] {
            let particle = decode_particle(&[0u8; 40], version).unwrap();
            assert_eq!(particle.version, version);
            let mut output = Cursor::new(Vec::new());
            write_particle(&particle, version, &mut output).unwrap();
            assert_eq!(output.into_inner(), vec![0u8; 40]);
        }
    }

    #[test]
    fn expands_legacy_bone_rows() {
        let mut data = vec![0u8; 40 + 48];
        data[20..22].copy_from_slice(&1u16.to_be_bytes());
        data[22..24].copy_from_slice(&48u16.to_be_bytes());

        let particle = decode_particle(&data, 0x73).unwrap();
        assert_eq!(particle.bones.len(), 1);
        let mut output = Cursor::new(Vec::new());
        write_particle(&particle, 0x73, &mut output).unwrap();
        assert_eq!(output.into_inner(), data);
    }

    #[test]
    fn expands_v77_force_field_rows() {
        let mut data = vec![0u8; 40 + 96];
        data[28..30].copy_from_slice(&1u16.to_be_bytes());
        data[30..32].copy_from_slice(&96u16.to_be_bytes());

        let particle = decode_particle(&data, 0x77).unwrap();
        assert_eq!(particle.forcefields.len(), 1);
        assert_eq!(particle.forcefields[0].data.len(), 0x70);
        let mut output = Cursor::new(Vec::new());
        write_particle(&particle, 0x77, &mut output).unwrap();
        assert_eq!(output.into_inner(), data);
    }

    #[test]
    fn accepts_v7b_geometry_cache() {
        let mut data = vec![0u8; 40 + 8 + 4];
        data[40..48].copy_from_slice(&4u64.to_be_bytes());

        let particle = decode_particle(&data, 0x7b).unwrap();
        assert_eq!(particle.version, 0x7b);
        let mut output = Cursor::new(Vec::new());
        write_particle(&particle, 0x7b, &mut output).unwrap();
        assert_eq!(output.into_inner(), data);
    }

    #[test]
    fn rejects_unknown_versions() {
        assert!(decode_particle(&[0u8; 40], 0x7c).is_err());
    }
}
