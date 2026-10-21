use std::io::Write;

use binrw::{
    io::{Read, Seek},
    BinRead, BinResult, BinWrite, ReadOptions, WriteOptions,
};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkCoord, NuccChunkModel, NuccChunkReference, NuccChunkType};

#[derive(Debug, Clone, Default)]
pub struct NuccChunkBillboard {
    pub version: u16,
    pub model: u32,
    pub flags: u32,
    pub frames: u16,
    pub reserved: u16,
    pub frame_milliseconds: u32,
    pub anm_data: NuccBillboardAnmData,
    pub reference: Option<NuccChunkReference>,
    pub coord: Option<NuccChunkCoord>,
    pub model_chunk: Option<Box<NuccChunkModel>>,
    pub raw_data: Option<Vec<u8>>,
}

pub type NuccBillboardReference = NuccChunkReference;

#[derive(Debug, Clone, Default)]
pub struct NuccBillboardAnmData {
    pub pos: Vec<[f32; 3]>,
    pub rot: Vec<f32>,
    pub scale: Vec<[f32; 2]>,
    pub opacity: Vec<f32>,
    pub uv1: Vec<[f32; 2]>,
    pub wh1: Vec<[f32; 2]>,
    pub blend_rate: Vec<f32>,
    pub blend_rate2: Vec<f32>,
    pub glare_rate: Vec<f32>,
    pub uv2: Vec<[f32; 2]>,
    pub wh2: Vec<[f32; 2]>,
    pub alpha_ref: Vec<f32>,
    pub outline_id: Vec<f32>,
}

impl BinRead for NuccChunkBillboard {
    type Args = (u16,);

    fn read_options<R: Read + Seek>(
        _reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkBillboard uses read_boxed",
        )
        .into())
    }
}

impl BinWrite for NuccChunkBillboard {
    type Args = (u16,);

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkBillboard uses write_boxed",
        )
        .into())
    }
}

impl NuccChunk for NuccChunkBillboard {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkBillboard
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if input.len() == 8 {
            let reference = NuccChunkReference {
                indices: [read_u32be(input, 0)?, read_u32be(input, 4)?],
            };
            let billboard = NuccChunkBillboard {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(billboard)));
        }

        if input.len() == 42 {
            if let Ok((_, chunk)) = NuccChunkCoord::read_boxed(input, version) {
                if let Ok(coord) = chunk.downcast::<NuccChunkCoord>() {
                    let billboard = NuccChunkBillboard {
                        version,
                        coord: Some(*coord),
                        ..Default::default()
                    };
                    return Ok((input.to_vec(), Box::new(billboard)));
                }
            }
        }

        if let Ok((_, chunk)) = NuccChunkModel::read_boxed(input, version) {
            if let Ok(model) = chunk.downcast::<NuccChunkModel>() {
                let billboard = NuccChunkBillboard {
                    version,
                    model_chunk: Some(model),
                    ..Default::default()
                };
                return Ok((input.to_vec(), Box::new(billboard)));
            }
        }

        match decode_billboard(input, version) {
            Ok(billboard) => Ok((input.to_vec(), Box::new(billboard))),
            Err(_) => {
                let billboard = NuccChunkBillboard {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(billboard)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut binrw::io::Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let billboard = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = billboard.reference {
            for index in reference.indices {
                output.write_all(&index.to_be_bytes())?;
            }
            return Ok(());
        }
        if let Some(raw) = &billboard.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        if let Some(coord) = &billboard.coord {
            NuccChunkCoord::write_boxed(Box::new(coord.clone()), output, version)?;
            return Ok(());
        }
        if let Some(model) = &billboard.model_chunk {
            NuccChunkModel::write_boxed(Box::new((**model).clone()), output, version)?;
            return Ok(());
        }
        output.write_all(&encode_billboard(&billboard, version)?)?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

fn decode_billboard(input: &[u8], version: u16) -> Result<NuccChunkBillboard> {
    if !matches!(version, 0x64..=0x7b) {
        return Err(NuccError::Unsupported(format!(
            "billboard version {version:#x}"
        )));
    }
    if input.len() < 16 {
        return Err(NuccError::Truncated);
    }

    let model = read_u32be(input, 0)?;
    let flags = read_u32be(input, 4)?;
    let frames = read_u16be(input, 8)?;
    let reserved = read_u16be(input, 10)?;
    let frame_milliseconds = read_u32be(input, 12)?;
    if frames == 0 || frame_milliseconds == 0 {
        return Err(NuccError::InvalidChunk("invalid billboard timing".into()));
    }

    let mut offset = 16;
    let anm_data = NuccBillboardAnmData {
        pos: read_vec3_channel(input, &mut offset, flags, 0, frames)?,
        rot: read_scalar_channel(input, &mut offset, flags, 1, frames)?,
        scale: read_vec2_channel(input, &mut offset, flags, 2, frames)?,
        opacity: read_scalar_channel(input, &mut offset, flags, 3, frames)?,
        uv1: read_vec2_channel(input, &mut offset, flags, 4, frames)?,
        wh1: read_vec2_channel(input, &mut offset, flags, 5, frames)?,
        blend_rate: read_scalar_channel(input, &mut offset, flags, 6, frames)?,
        blend_rate2: read_scalar_channel(input, &mut offset, flags, 7, frames)?,
        glare_rate: read_scalar_channel(input, &mut offset, flags, 8, frames)?,
        uv2: read_vec2_channel(input, &mut offset, flags, 9, frames)?,
        wh2: read_vec2_channel(input, &mut offset, flags, 10, frames)?,
        alpha_ref: read_scalar_channel(input, &mut offset, flags, 11, frames)?,
        outline_id: read_scalar_channel(input, &mut offset, flags, 12, frames)?,
    };

    if offset != input.len() {
        return Err(NuccError::InvalidChunk(
            "unconsumed billboard payload".into(),
        ));
    }

    Ok(NuccChunkBillboard {
        version,
        model,
        flags,
        frames,
        reserved,
        frame_milliseconds,
        anm_data,
        reference: None,
        coord: None,
        model_chunk: None,
        raw_data: None,
    })
}

fn encode_billboard(
    billboard: &NuccChunkBillboard,
    version: u16,
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !matches!(version, 0x64..=0x7b) {
        return Err(Box::new(NuccError::Unsupported(format!(
            "billboard version {version:#x}"
        ))));
    }
    if billboard.frames == 0 || billboard.frame_milliseconds == 0 {
        return Err(Box::new(NuccError::InvalidChunk(
            "invalid billboard timing".into(),
        )));
    }

    let mut output = Vec::new();
    write_u32(&mut output, billboard.model);
    write_u32(&mut output, billboard.flags);
    write_u16(&mut output, billboard.frames);
    write_u16(&mut output, billboard.reserved);
    write_u32(&mut output, billboard.frame_milliseconds);

    write_vec3_channel(
        &mut output,
        billboard.flags,
        0,
        billboard.frames,
        &billboard.anm_data.pos,
    )?;
    write_scalar_channel(
        &mut output,
        billboard.flags,
        1,
        billboard.frames,
        &billboard.anm_data.rot,
    )?;
    write_vec2_channel(
        &mut output,
        billboard.flags,
        2,
        billboard.frames,
        &billboard.anm_data.scale,
    )?;
    write_scalar_channel(
        &mut output,
        billboard.flags,
        3,
        billboard.frames,
        &billboard.anm_data.opacity,
    )?;
    write_vec2_channel(
        &mut output,
        billboard.flags,
        4,
        billboard.frames,
        &billboard.anm_data.uv1,
    )?;
    write_vec2_channel(
        &mut output,
        billboard.flags,
        5,
        billboard.frames,
        &billboard.anm_data.wh1,
    )?;
    write_scalar_channel(
        &mut output,
        billboard.flags,
        6,
        billboard.frames,
        &billboard.anm_data.blend_rate,
    )?;
    write_scalar_channel(
        &mut output,
        billboard.flags,
        7,
        billboard.frames,
        &billboard.anm_data.blend_rate2,
    )?;
    write_scalar_channel(
        &mut output,
        billboard.flags,
        8,
        billboard.frames,
        &billboard.anm_data.glare_rate,
    )?;
    write_vec2_channel(
        &mut output,
        billboard.flags,
        9,
        billboard.frames,
        &billboard.anm_data.uv2,
    )?;
    write_vec2_channel(
        &mut output,
        billboard.flags,
        10,
        billboard.frames,
        &billboard.anm_data.wh2,
    )?;
    write_scalar_channel(
        &mut output,
        billboard.flags,
        11,
        billboard.frames,
        &billboard.anm_data.alpha_ref,
    )?;
    write_scalar_channel(
        &mut output,
        billboard.flags,
        12,
        billboard.frames,
        &billboard.anm_data.outline_id,
    )?;
    Ok(output)
}

fn channel_count(flags: u32, field: u32, frames: u16) -> Result<usize> {
    match (flags >> (1 + field * 2)) & 3 {
        0 => Ok(0),
        1 => Ok(1),
        2 => Ok(frames as usize),
        _ => Err(NuccError::InvalidChunk(
            "invalid billboard channel mode".into(),
        )),
    }
}

fn read_scalar_channel(
    input: &[u8],
    offset: &mut usize,
    flags: u32,
    field: u32,
    frames: u16,
) -> Result<Vec<f32>> {
    let count = channel_count(flags, field, frames)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let value = read_f32(input, *offset)?;
        if !value.is_finite() {
            return Err(NuccError::InvalidChunk("non-finite billboard key".into()));
        }
        values.push(value);
        *offset += 4;
    }
    Ok(values)
}

fn read_vec2_channel(
    input: &[u8],
    offset: &mut usize,
    flags: u32,
    field: u32,
    frames: u16,
) -> Result<Vec<[f32; 2]>> {
    let count = channel_count(flags, field, frames)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let value = [read_f32(input, *offset)?, read_f32(input, *offset + 4)?];
        if !value.iter().all(|value| value.is_finite()) {
            return Err(NuccError::InvalidChunk("non-finite billboard key".into()));
        }
        values.push(value);
        *offset += 8;
    }
    Ok(values)
}

fn read_vec3_channel(
    input: &[u8],
    offset: &mut usize,
    flags: u32,
    field: u32,
    frames: u16,
) -> Result<Vec<[f32; 3]>> {
    let count = channel_count(flags, field, frames)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let value = [
            read_f32(input, *offset)?,
            read_f32(input, *offset + 4)?,
            read_f32(input, *offset + 8)?,
        ];
        if !value.iter().all(|value| value.is_finite()) {
            return Err(NuccError::InvalidChunk("non-finite billboard key".into()));
        }
        values.push(value);
        *offset += 12;
    }
    Ok(values)
}

fn write_scalar_channel(
    output: &mut Vec<u8>,
    flags: u32,
    field: u32,
    frames: u16,
    values: &[f32],
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if values.len() != channel_count(flags, field, frames)? {
        return Err(Box::new(NuccError::InvalidChunk(
            "billboard channel count mismatch".into(),
        )));
    }
    for value in values {
        write_u32(output, value.to_bits());
    }
    Ok(())
}

fn write_vec2_channel(
    output: &mut Vec<u8>,
    flags: u32,
    field: u32,
    frames: u16,
    values: &[[f32; 2]],
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if values.len() != channel_count(flags, field, frames)? {
        return Err(Box::new(NuccError::InvalidChunk(
            "billboard channel count mismatch".into(),
        )));
    }
    for value in values {
        write_u32(output, value[0].to_bits());
        write_u32(output, value[1].to_bits());
    }
    Ok(())
}

fn write_vec3_channel(
    output: &mut Vec<u8>,
    flags: u32,
    field: u32,
    frames: u16,
    values: &[[f32; 3]],
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if values.len() != channel_count(flags, field, frames)? {
        return Err(Box::new(NuccError::InvalidChunk(
            "billboard channel count mismatch".into(),
        )));
    }
    for value in values {
        write_u32(output, value[0].to_bits());
        write_u32(output, value[1].to_bits());
        write_u32(output, value[2].to_bits());
    }
    Ok(())
}

fn read_u16be(data: &[u8], at: usize) -> Result<u16> {
    let bytes = data.get(at..at + 2).ok_or(NuccError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32be(data: &[u8], at: usize) -> Result<u32> {
    let bytes = data.get(at..at + 4).ok_or(NuccError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_f32(data: &[u8], at: usize) -> Result<f32> {
    Ok(f32::from_bits(read_u32be(data, at)?))
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_constant_channels() {
        let data = minimal_billboard();
        let billboard = decode_billboard(&data, 0x79).unwrap();
        assert_eq!(billboard.model, 1);
        assert_eq!(billboard.frames, 2);
        assert_eq!(billboard.anm_data.pos, vec![[1.0, 2.0, 3.0]]);
        assert_eq!(billboard.anm_data.rot, vec![0.5]);
        assert!(billboard.anm_data.scale.is_empty());
        assert_eq!(encode_billboard(&billboard, 0x79).unwrap(), data);
    }

    #[test]
    fn parses_reference_stub() {
        let (_, chunk) = NuccChunkBillboard::read_boxed(&[0, 0, 0, 12, 0, 0, 0, 0], 0x79).unwrap();
        let billboard = chunk
            .downcast_ref::<NuccChunkBillboard>()
            .expect("billboard");
        assert_eq!(
            billboard.reference,
            Some(NuccChunkReference { indices: [12, 0] })
        );
        assert!(!billboard.is_opaque());
    }

    fn minimal_billboard() -> Vec<u8> {
        let mut data = Vec::new();
        write_u32(&mut data, 1);
        write_u32(&mut data, (1 << 1) | (1 << (1 + 2)));
        write_u16(&mut data, 2);
        write_u16(&mut data, 0);
        write_u32(&mut data, 33);
        for value in [1.0f32, 2.0, 3.0] {
            write_u32(&mut data, value.to_bits());
        }
        write_u32(&mut data, 0.5f32.to_bits());
        data
    }
}
