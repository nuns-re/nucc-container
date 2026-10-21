use std::io::Write;

use binrw::{
    io::{Read, Seek},
    BinRead, BinResult, BinWrite, ReadOptions, WriteOptions,
};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkCoord, NuccChunkReference, NuccChunkType};

#[derive(Debug, Clone, Default)]
pub struct NuccChunkTrail {
    pub version: u16,
    pub header: NuccXfbinTrailDataHeader,
    pub gen_data: Vec<NuccXfbinTrailGenDataBinary>,
    pub billboard_data: Vec<NuccXfbinTrailBillboardData>,
    pub pos_data: Vec<NuccXfbinTrailPosData>,
    pub ff_data: Vec<NuccXfbinTrailFFData>,
    pub timelines: Vec<NuccTrailTimelineParam>,
    pub reference: Option<NuccChunkReference>,
    pub coord: Option<NuccChunkCoord>,
    pub raw_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NuccXfbinTrailSection {
    pub position: u32,
    pub count: u16,
    pub size: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NuccXfbinTrailDataHeader {
    pub gen: NuccXfbinTrailSection,
    pub billboard: NuccXfbinTrailSection,
    pub pos: NuccXfbinTrailSection,
    pub ff: NuccXfbinTrailSection,
    pub timeline: NuccXfbinTrailSection,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NuccXfbinTrailGenDataBinary {
    pub animation: u32,
    pub entry_index: u32,
    pub reserved0: [u8; 12],
    pub length: u32,
    pub subdivisions: u32,
    pub flags: u16,
    pub reserved1: [u8; 2],
    pub colors: [[f32; 4]; 3],
    pub midpoint: f32,
    pub reserved2: [u8; 12],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NuccXfbinTrailBillboardData {
    pub billboard: u32,
    pub entry_index: u32,
    pub reserved: [u8; 24],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NuccXfbinTrailPosDataBinary {
    pub coordinate: u32,
    pub entry_index: u32,
    pub reserved0: [u8; 24],
    pub clump: u32,
    pub reserved1: [u8; 12],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NuccXfbinTrailPosData0078Binary {
    pub coordinate: u32,
    pub entry_index: u32,
    pub reserved: [u8; 24],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuccXfbinTrailPosData {
    Binary(NuccXfbinTrailPosDataBinary),
    Binary0078(NuccXfbinTrailPosData0078Binary),
}

impl NuccXfbinTrailPosData {
    pub fn coordinate(&self) -> u32 {
        match self {
            Self::Binary(data) => data.coordinate,
            Self::Binary0078(data) => data.coordinate,
        }
    }

    pub fn entry_index(&self) -> u32 {
        match self {
            Self::Binary(data) => data.entry_index,
            Self::Binary0078(data) => data.entry_index,
        }
    }

    pub fn clump(&self) -> Option<u32> {
        match self {
            Self::Binary(data) => Some(data.clump),
            Self::Binary0078(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NuccXfbinTrailFFDataBinary {
    pub coordinate: u32,
    pub reserved: [u8; 60],
}

impl Default for NuccXfbinTrailFFDataBinary {
    fn default() -> Self {
        Self {
            coordinate: 0,
            reserved: [0; 60],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NuccXfbinTrailFFData0078Binary {
    pub coordinate: u32,
    pub reserved: [u8; 44],
}

impl Default for NuccXfbinTrailFFData0078Binary {
    fn default() -> Self {
        Self {
            coordinate: 0,
            reserved: [0; 44],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuccXfbinTrailFFData {
    Binary(NuccXfbinTrailFFDataBinary),
    Binary0078(NuccXfbinTrailFFData0078Binary),
}

impl NuccXfbinTrailFFData {
    pub fn coordinate(&self) -> u32 {
        match self {
            Self::Binary(data) => data.coordinate,
            Self::Binary0078(data) => data.coordinate,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NuccTrailTimelineParam {
    pub index: u32,
    pub frame_count: u32,
    pub frames: Vec<u32>,
}

impl BinRead for NuccChunkTrail {
    type Args = (u16,);

    fn read_options<R: Read + Seek>(
        _reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkTrail uses read_boxed",
        )
        .into())
    }
}

impl BinWrite for NuccChunkTrail {
    type Args = (u16,);

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkTrail uses write_boxed",
        )
        .into())
    }
}

impl NuccChunk for NuccChunkTrail {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkTrail
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if input.len() == 8 {
            let reference = NuccChunkReference {
                indices: [read_u32be(input, 0)?, read_u32be(input, 4)?],
            };
            let trail = NuccChunkTrail {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(trail)));
        }

        if input.len() == 42 {
            if let Ok((_, chunk)) = NuccChunkCoord::read_boxed(input, version) {
                if let Ok(coord) = chunk.downcast::<NuccChunkCoord>() {
                    let trail = NuccChunkTrail {
                        version,
                        coord: Some(*coord),
                        ..Default::default()
                    };
                    return Ok((input.to_vec(), Box::new(trail)));
                }
            }
        }

        match decode_trail(input, version) {
            Ok(trail) => Ok((input.to_vec(), Box::new(trail))),
            Err(_) => {
                let trail = NuccChunkTrail {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(trail)))
            }
        }
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut binrw::io::Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let trail = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = trail.reference {
            for index in reference.indices {
                output.write_all(&index.to_be_bytes())?;
            }
            return Ok(());
        }
        if let Some(raw) = &trail.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        if let Some(coord) = &trail.coord {
            NuccChunkCoord::write_boxed(Box::new(coord.clone()), output, version)?;
            return Ok(());
        }
        output.write_all(&encode_trail(&trail, version)?)?;
        Ok(())
    }
}

fn decode_trail(input: &[u8], version: u16) -> Result<NuccChunkTrail> {
    if !matches!(version, 0x73..=0x7b) {
        return Err(NuccError::Unsupported(format!(
            "trail version {version:#x}"
        )));
    }
    if input.len() < 40 {
        return Err(NuccError::Truncated);
    }

    let mut sections = [NuccXfbinTrailSection::default(); 5];
    for (index, section) in sections.iter_mut().enumerate() {
        let at = index * 8;
        section.position = read_u32be(input, at)?;
        section.count = read_u16be(input, at + 4)?;
        section.size = read_u16be(input, at + 6)?;
    }
    let header = NuccXfbinTrailDataHeader {
        gen: sections[0],
        billboard: sections[1],
        pos: sections[2],
        ff: sections[3],
        timeline: sections[4],
    };
    if header.gen.position != 40 {
        return Err(NuccError::InvalidIndex(
            "invalid trail header offset".into(),
        ));
    }

    let legacy = version < 0x79;
    let pos_stride = if legacy { 32 } else { 48 };
    let ff_stride = if legacy { 48 } else { 64 };
    let strides = [96usize, 32, pos_stride, ff_stride];
    for (index, stride) in strides.into_iter().enumerate() {
        let section = sections[index];
        if section.count as usize * stride != section.size as usize {
            return Err(NuccError::InvalidIndex(format!(
                "invalid trail section {index} size"
            )));
        }
        let end = section.position as usize + section.size as usize;
        if end > input.len() {
            return Err(NuccError::Truncated);
        }
    }

    for pair in sections.windows(2) {
        if pair[0].position > pair[1].position {
            return Err(NuccError::InvalidIndex(
                "trail sections are out of order".into(),
            ));
        }
    }

    let gen_data = (0..sections[0].count as usize)
        .map(|index| read_gen_data(input, sections[0].position as usize + index * 96))
        .collect::<Result<Vec<_>>>()?;
    let billboard_data = (0..sections[1].count as usize)
        .map(|index| read_billboard_data(input, sections[1].position as usize + index * 32))
        .collect::<Result<Vec<_>>>()?;
    let pos_data = (0..sections[2].count as usize)
        .map(|index| {
            read_pos_data(
                input,
                sections[2].position as usize + index * pos_stride,
                legacy,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let ff_data = (0..sections[3].count as usize)
        .map(|index| {
            read_ff_data(
                input,
                sections[3].position as usize + index * ff_stride,
                legacy,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let timelines = read_timelines(
        input,
        sections[4].position as usize,
        sections[4].count as usize,
    )?;

    Ok(NuccChunkTrail {
        version,
        header,
        gen_data,
        billboard_data,
        pos_data,
        ff_data,
        timelines,
        reference: None,
        coord: None,
        raw_data: None,
    })
}

fn read_gen_data(data: &[u8], at: usize) -> Result<NuccXfbinTrailGenDataBinary> {
    let mut colors = [[0.0; 4]; 3];
    for (color_index, color) in colors.iter_mut().enumerate() {
        for (channel, value) in color.iter_mut().enumerate() {
            *value = read_f32(data, at + 32 + color_index * 16 + channel * 4)?;
        }
    }

    Ok(NuccXfbinTrailGenDataBinary {
        animation: read_u32be(data, at)?,
        entry_index: read_u32be(data, at + 4)?,
        reserved0: read_array(data, at + 8)?,
        length: read_u32be(data, at + 20)?,
        subdivisions: read_u32be(data, at + 24)?,
        flags: read_u16be(data, at + 28)?,
        reserved1: read_array(data, at + 30)?,
        colors,
        midpoint: read_f32(data, at + 80)?,
        reserved2: read_array(data, at + 84)?,
    })
}

fn read_billboard_data(data: &[u8], at: usize) -> Result<NuccXfbinTrailBillboardData> {
    Ok(NuccXfbinTrailBillboardData {
        billboard: read_u32be(data, at)?,
        entry_index: read_u32be(data, at + 4)?,
        reserved: read_array(data, at + 8)?,
    })
}

fn read_pos_data(data: &[u8], at: usize, legacy: bool) -> Result<NuccXfbinTrailPosData> {
    if legacy {
        Ok(NuccXfbinTrailPosData::Binary0078(
            NuccXfbinTrailPosData0078Binary {
                coordinate: read_u32be(data, at)?,
                entry_index: read_u32be(data, at + 4)?,
                reserved: read_array(data, at + 8)?,
            },
        ))
    } else {
        Ok(NuccXfbinTrailPosData::Binary(NuccXfbinTrailPosDataBinary {
            coordinate: read_u32be(data, at)?,
            entry_index: read_u32be(data, at + 4)?,
            reserved0: read_array(data, at + 8)?,
            clump: read_u32be(data, at + 32)?,
            reserved1: read_array(data, at + 36)?,
        }))
    }
}

fn read_ff_data(data: &[u8], at: usize, legacy: bool) -> Result<NuccXfbinTrailFFData> {
    if legacy {
        Ok(NuccXfbinTrailFFData::Binary0078(
            NuccXfbinTrailFFData0078Binary {
                coordinate: read_u32be(data, at)?,
                reserved: read_array(data, at + 4)?,
            },
        ))
    } else {
        Ok(NuccXfbinTrailFFData::Binary(NuccXfbinTrailFFDataBinary {
            coordinate: read_u32be(data, at)?,
            reserved: read_array(data, at + 4)?,
        }))
    }
}

fn read_timelines(data: &[u8], start: usize, count: usize) -> Result<Vec<NuccTrailTimelineParam>> {
    let mut timelines = Vec::with_capacity(count);
    let mut at = start;
    for _ in 0..count {
        let index = read_u32be(data, at + 4)?;
        let frame_count = read_u32be(data, at + 16)? as usize;
        if frame_count > 65536 || frame_count > data.len().saturating_sub(at + 20) / 4 {
            return Err(NuccError::InvalidIndex(
                "trail timeline exceeds payload".into(),
            ));
        }
        let frames = (0..frame_count)
            .map(|frame| read_u32be(data, at + 20 + frame * 4))
            .collect::<Result<Vec<_>>>()?;
        timelines.push(NuccTrailTimelineParam {
            index,
            frame_count: frame_count as u32,
            frames,
        });
        at += (20 + frame_count * 4 + 7) & !7;
        if at > data.len() {
            return Err(NuccError::Truncated);
        }
    }

    let padding = &data[at..];
    if padding.len() >= 8 || padding.iter().any(|byte| *byte != 0 && *byte != 0xcd) {
        return Err(NuccError::InvalidIndex(
            "unconsumed trail timeline data".into(),
        ));
    }
    Ok(timelines)
}

fn encode_trail(
    trail: &NuccChunkTrail,
    version: u16,
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !matches!(version, 0x73..=0x7b) {
        return Err(Box::new(NuccError::Unsupported(format!(
            "trail version {version:#x}"
        ))));
    }
    let legacy = version < 0x79;
    let pos_stride = if legacy { 32 } else { 48 };
    let ff_stride = if legacy { 48 } else { 64 };
    let gen_size = trail.gen_data.len() * 96;
    let billboard_size = trail.billboard_data.len() * 32;
    let pos_size = trail.pos_data.len() * pos_stride;
    let ff_size = trail.ff_data.len() * ff_stride;
    let timeline_offset = 40 + gen_size + billboard_size + pos_size + ff_size;

    let mut timeline_bytes = Vec::new();
    for timeline in &trail.timelines {
        let record_size = (20 + timeline.frames.len() * 4 + 7) & !7;
        let mut record = vec![0u8; record_size];
        record[4..8].copy_from_slice(&timeline.index.to_be_bytes());
        record[16..20].copy_from_slice(&(timeline.frames.len() as u32).to_be_bytes());
        for (index, frame) in timeline.frames.iter().enumerate() {
            record[20 + index * 4..24 + index * 4].copy_from_slice(&frame.to_be_bytes());
        }
        timeline_bytes.extend_from_slice(&record);
    }

    let mut output = Vec::new();
    write_section(&mut output, 40, trail.gen_data.len(), gen_size)?;
    write_section(
        &mut output,
        40 + gen_size,
        trail.billboard_data.len(),
        billboard_size,
    )?;
    write_section(
        &mut output,
        40 + gen_size + billboard_size,
        trail.pos_data.len(),
        pos_size,
    )?;
    write_section(
        &mut output,
        40 + gen_size + billboard_size + pos_size,
        trail.ff_data.len(),
        ff_size,
    )?;
    write_section(
        &mut output,
        timeline_offset,
        trail.timelines.len(),
        timeline_bytes.len(),
    )?;

    for gen in &trail.gen_data {
        write_u32(&mut output, gen.animation);
        write_u32(&mut output, gen.entry_index);
        output.extend_from_slice(&gen.reserved0);
        write_u32(&mut output, gen.length);
        write_u32(&mut output, gen.subdivisions);
        write_u16(&mut output, gen.flags);
        output.extend_from_slice(&gen.reserved1);
        for color in gen.colors {
            for channel in color {
                write_u32(&mut output, channel.to_bits());
            }
        }
        write_u32(&mut output, gen.midpoint.to_bits());
        output.extend_from_slice(&gen.reserved2);
    }
    for billboard in &trail.billboard_data {
        write_u32(&mut output, billboard.billboard);
        write_u32(&mut output, billboard.entry_index);
        output.extend_from_slice(&billboard.reserved);
    }
    for pos in &trail.pos_data {
        match pos {
            NuccXfbinTrailPosData::Binary(pos) => {
                write_u32(&mut output, pos.coordinate);
                write_u32(&mut output, pos.entry_index);
                output.extend_from_slice(&pos.reserved0);
                if legacy {
                    output.truncate(output.len() - 16);
                } else {
                    write_u32(&mut output, pos.clump);
                    output.extend_from_slice(&pos.reserved1);
                }
            }
            NuccXfbinTrailPosData::Binary0078(pos) => {
                write_u32(&mut output, pos.coordinate);
                write_u32(&mut output, pos.entry_index);
                output.extend_from_slice(&pos.reserved);
                if !legacy {
                    output.extend_from_slice(&[0u8; 16]);
                }
            }
        }
    }
    for ff in &trail.ff_data {
        match ff {
            NuccXfbinTrailFFData::Binary(ff) => {
                write_u32(&mut output, ff.coordinate);
                output.extend_from_slice(&ff.reserved);
                if legacy {
                    output.truncate(output.len() - 16);
                }
            }
            NuccXfbinTrailFFData::Binary0078(ff) => {
                write_u32(&mut output, ff.coordinate);
                output.extend_from_slice(&ff.reserved);
                if !legacy {
                    output.extend_from_slice(&[0u8; 16]);
                }
            }
        }
    }
    output.extend_from_slice(&timeline_bytes);
    Ok(output)
}

fn write_section(
    output: &mut Vec<u8>,
    position: usize,
    count: usize,
    size: usize,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let count = u16::try_from(count)?;
    let size = u16::try_from(size)?;
    write_u32(output, u32::try_from(position)?);
    write_u16(output, count);
    write_u16(output, size);
    Ok(())
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
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

fn read_array<const N: usize>(data: &[u8], at: usize) -> Result<[u8; N]> {
    data.get(at..at + N)
        .ok_or(NuccError::Truncated)?
        .try_into()
        .map_err(|_| NuccError::Truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_all_trail_versions() {
        for version in 0x73..=0x7b {
            let mut data = vec![0u8; 40];
            for section in 0..5 {
                data[section * 8..section * 8 + 4].copy_from_slice(&40u32.to_be_bytes());
            }
            let trail = decode_trail(&data, version).unwrap();
            assert_eq!(trail.version, version);
            let encoded = encode_trail(&trail, version).unwrap();
            assert_eq!(encoded, data);
        }
    }

    #[test]
    fn roundtrips_current_trail_data() {
        let mut data = vec![0u8; 360];
        for (index, position, count, size) in [
            (0, 40u32, 1u16, 96u16),
            (1, 136, 1, 32),
            (2, 168, 2, 96),
            (3, 264, 1, 64),
            (4, 328, 1, 32),
        ] {
            data[index * 8..index * 8 + 4].copy_from_slice(&position.to_be_bytes());
            data[index * 8 + 4..index * 8 + 6].copy_from_slice(&count.to_be_bytes());
            data[index * 8 + 6..index * 8 + 8].copy_from_slice(&size.to_be_bytes());
        }

        data[40..44].copy_from_slice(&1u32.to_be_bytes());
        data[44..48].copy_from_slice(&2u32.to_be_bytes());
        data[60..64].copy_from_slice(&5u32.to_be_bytes());
        data[64..68].copy_from_slice(&3u32.to_be_bytes());
        data[68..70].copy_from_slice(&0x13u16.to_be_bytes());
        data[72..76].copy_from_slice(&1.0f32.to_be_bytes());
        data[120..124].copy_from_slice(&0.5f32.to_be_bytes());
        data[136..140].copy_from_slice(&3u32.to_be_bytes());
        data[140..144].copy_from_slice(&2u32.to_be_bytes());
        data[168..172].copy_from_slice(&4u32.to_be_bytes());
        data[172..176].copy_from_slice(&2u32.to_be_bytes());
        data[200..204].copy_from_slice(&6u32.to_be_bytes());
        data[216..220].copy_from_slice(&5u32.to_be_bytes());
        data[220..224].copy_from_slice(&2u32.to_be_bytes());
        data[248..252].copy_from_slice(&7u32.to_be_bytes());
        data[264..268].copy_from_slice(&8u32.to_be_bytes());
        data[332..336].copy_from_slice(&2u32.to_be_bytes());
        data[344..348].copy_from_slice(&2u32.to_be_bytes());
        data[348..352].copy_from_slice(&0x21u32.to_be_bytes());
        data[352..356].copy_from_slice(&67u32.to_be_bytes());

        let trail = decode_trail(&data, 0x79).unwrap();
        assert_eq!(trail.gen_data[0].length, 5);
        assert_eq!(trail.gen_data[0].subdivisions, 3);
        assert_eq!(trail.pos_data[0].coordinate(), 4);
        assert_eq!(trail.pos_data[0].clump(), Some(6));
        assert_eq!(trail.ff_data[0].coordinate(), 8);
        assert_eq!(trail.timelines[0].frames, [0x21, 67]);
        assert_eq!(encode_trail(&trail, 0x79).unwrap(), data);
    }

    #[test]
    fn parses_legacy_and_new_position_rows() {
        let mut data = vec![0u8; 40 + 32 * 2];
        for section in 0..3 {
            data[section * 8..section * 8 + 4].copy_from_slice(&40u32.to_be_bytes());
        }
        data[16..20].copy_from_slice(&40u32.to_be_bytes());
        data[20..22].copy_from_slice(&2u16.to_be_bytes());
        data[22..24].copy_from_slice(&64u16.to_be_bytes());
        data[24..28].copy_from_slice(&104u32.to_be_bytes());
        data[32..36].copy_from_slice(&104u32.to_be_bytes());
        let legacy = decode_trail(&data, 0x75).unwrap();
        assert_eq!(legacy.pos_data.len(), 2);

        let mut data = vec![0u8; 40 + 48 * 2];
        for section in 0..3 {
            data[section * 8..section * 8 + 4].copy_from_slice(&40u32.to_be_bytes());
        }
        data[16..20].copy_from_slice(&40u32.to_be_bytes());
        data[20..22].copy_from_slice(&2u16.to_be_bytes());
        data[22..24].copy_from_slice(&96u16.to_be_bytes());
        data[24..28].copy_from_slice(&136u32.to_be_bytes());
        data[32..36].copy_from_slice(&136u32.to_be_bytes());
        let current = decode_trail(&data, 0x79).unwrap();
        assert_eq!(current.pos_data.len(), 2);
    }
}
