use nucc_core::{NuccError, Result};

const HEADER_SIZE: usize = 0x18;
const TONE_ID: [u8; 4] = *b"TONE";
const PACK_ID: [u8; 4] = *b"PACK";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nus3Chunk {
    pub id: [u8; 4],
    pub offset: usize,
    pub size: usize,
}

impl Nus3Chunk {
    pub fn id_str(&self) -> String {
        String::from_utf8_lossy(&self.id).into_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nus3Codec {
    Idsp,
    Ivag,
    Bnsf,
    Riff,
    RiffXma2,
    RiffEncrypted,
    Opus,
    Unknown([u8; 4]),
}

impl Nus3Codec {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Idsp => "idsp",
            Self::Ivag => "ivag",
            Self::Bnsf => "bnsf",
            Self::Riff => "wav",
            Self::RiffXma2 | Self::RiffEncrypted => "xma",
            Self::Opus => "opus",
            Self::Unknown(_) => "bin",
        }
    }

    fn from_stream(data: &[u8], offset: usize) -> Result<Self> {
        let magic = read_u32be(data, offset)?;

        Ok(match magic {
            0x4944_5350 => Self::Idsp,
            0x4956_4147 => Self::Ivag,
            0x424e_5346 => Self::Bnsf,
            0x5249_4646 => {
                if read_u16le(data, offset + 0x14).unwrap_or(0) == 0x0166 {
                    Self::RiffXma2
                } else {
                    Self::Riff
                }
            }
            0x552a_af17 => Self::RiffEncrypted,
            0x4f50_5553 => Self::Opus,
            _ => Self::Unknown(magic.to_be_bytes()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nus3Tone {
    pub index: u32,
    pub name: String,
    pub header_offset: usize,
    pub header_size: usize,
    pub tone_type: u32,
    pub flags1: u8,
    pub flags2: u8,
    pub stream_offset: usize,
    pub stream_size: usize,
    pub codec: Nus3Codec,
}

#[derive(Debug, Clone)]
pub struct Nus3Bank {
    data: Vec<u8>,
    pub file_size: usize,
    pub chunks: Vec<Nus3Chunk>,
    pub tones: Vec<Nus3Tone>,
}

impl Nus3Bank {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(NuccError::Truncated);
        }

        if &data[0..4] != b"NUS3" || &data[8..12] != b"BANK" || &data[12..16] != b"TOC " {
            return Err(NuccError::InvalidHeader);
        }

        let file_size = read_u32le(data, 0x04)? as usize;
        let toc_size = read_u32le(data, 0x10)? as usize;
        let chunk_count = read_u32le(data, 0x14)? as usize;
        let toc_end = add(
            HEADER_SIZE,
            chunk_count.checked_mul(8).ok_or(NuccError::Overflow)?,
        )?;

        if toc_end > data.len() {
            return Err(NuccError::InvalidIndex("chunk table exceeds file".into()));
        }

        let mut cursor = add(0x14, toc_size)?;
        if cursor < toc_end || cursor > data.len() {
            return Err(NuccError::InvalidIndex("invalid TOC size".into()));
        }

        let mut chunks = Vec::with_capacity(chunk_count);
        for i in 0..chunk_count {
            let entry = add(HEADER_SIZE, i.checked_mul(8).ok_or(NuccError::Overflow)?)?;
            let id: [u8; 4] = data
                .get(entry..entry + 4)
                .ok_or(NuccError::Truncated)?
                .try_into()
                .map_err(|_| NuccError::Truncated)?;
            let size = read_u32le(data, entry + 4)? as usize;
            let offset = add(cursor, 8)?;
            let end = add(offset, size)?;

            if end > data.len() {
                return Err(NuccError::InvalidChunk(format!(
                    "{} chunk exceeds file: offset={offset:#x} size={size:#x} end={end:#x} len={:#x} cursor={cursor:#x}",
                    String::from_utf8_lossy(&id),
                    data.len()
                )));
            }

            chunks.push(Nus3Chunk { id, offset, size });
            cursor = end;
        }

        let tone_chunk = chunks
            .iter()
            .find(|chunk| chunk.id == TONE_ID)
            .ok_or_else(|| NuccError::InvalidChunk("missing TONE chunk".into()))?;
        let pack_chunk = chunks
            .iter()
            .find(|chunk| chunk.id == PACK_ID)
            .ok_or_else(|| NuccError::InvalidChunk("missing PACK chunk".into()))?;

        let tone_end = add(tone_chunk.offset, tone_chunk.size)?;
        let entries = read_u32le(data, tone_chunk.offset)? as usize;
        let entry_size = entries.checked_mul(8).ok_or(NuccError::Overflow)?;
        let entries_end = add(add(tone_chunk.offset, 4)?, entry_size)?;
        if entries_end > tone_end {
            return Err(NuccError::InvalidChunk(format!(
                "TONE table exceeds chunk: entries_end={entries_end:#x} tone_end={tone_end:#x} offset={:#x} size={:#x} entries={entries}",
                tone_chunk.offset, tone_chunk.size
            )));
        }

        let mut tones = Vec::new();
        for index in 0..entries {
            let entry = add(add(tone_chunk.offset, 4)?, index * 8)?;
            let header_offset = add(tone_chunk.offset, read_u32le(data, entry)? as usize)?;
            let header_size = read_u32le(data, entry + 4)? as usize;

            if header_size <= 0x0c {
                continue;
            }
            if add(header_offset, header_size)? > tone_end {
                return Err(NuccError::InvalidChunk(
                    "tone header exceeds TONE chunk".into(),
                ));
            }

            if let Some(tone) = parse_tone(
                data,
                tone_chunk,
                pack_chunk,
                index,
                header_offset,
                header_size,
            )? {
                tones.push(tone);
            }
        }

        Ok(Self {
            data: data.to_vec(),
            file_size,
            chunks,
            tones,
        })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn chunk(&self, id: &[u8; 4]) -> Option<&Nus3Chunk> {
        self.chunks.iter().find(|chunk| chunk.id == *id)
    }

    pub fn tone_stream(&self, index: usize) -> Option<&[u8]> {
        let tone = self.tones.get(index)?;
        let end = tone.stream_offset.checked_add(tone.stream_size)?;
        self.data.get(tone.stream_offset..end)
    }

    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

fn parse_tone(
    data: &[u8],
    tone_chunk: &Nus3Chunk,
    pack_chunk: &Nus3Chunk,
    index: usize,
    header_offset: usize,
    header_size: usize,
) -> Result<Option<Nus3Tone>> {
    let tone_type = read_u32le(data, header_offset)?;
    let flags1 = read_u8(data, header_offset + 0x06)?;
    let flags2 = read_u8(data, header_offset + 0x07)?;
    let tone_end = add(tone_chunk.offset, tone_chunk.size)?;

    let mut cursor = add(header_offset, 8)?;
    if flags2 & 0x80 != 0 {
        cursor = add(cursor, 4)?;
    }

    let name_size = read_u8(data, cursor)? as usize;
    let name_offset = add(cursor, 1)?;
    let name_end = add(name_offset, name_size)?;
    if name_end > tone_end {
        return Err(NuccError::InvalidChunk(
            "tone name exceeds TONE chunk".into(),
        ));
    }
    let name = String::from_utf8_lossy(&data[name_offset..name_end])
        .trim_end_matches('\0')
        .to_owned();

    cursor = add(cursor, align(1 + name_size, 4))?;
    if add(cursor, 0x10)? > tone_end {
        return Err(NuccError::InvalidChunk(
            "tone record exceeds TONE chunk".into(),
        ));
    }
    if read_u32le(data, cursor + 4)? != 8 {
        return Ok(None);
    }

    let stream_offset = add(pack_chunk.offset, read_u32le(data, cursor + 8)? as usize)?;
    let stream_size = read_u32le(data, cursor + 0x0c)? as usize;
    if stream_size == 0 {
        return Ok(None);
    }

    let stream_end = add(stream_offset, stream_size)?;
    if stream_end > data.len() {
        return Err(NuccError::InvalidChunk("tone stream exceeds file".into()));
    }

    let codec = Nus3Codec::from_stream(data, stream_offset)?;
    Ok(Some(Nus3Tone {
        index: index as u32,
        name,
        header_offset,
        header_size,
        tone_type,
        flags1,
        flags2,
        stream_offset,
        stream_size,
        codec,
    }))
}

fn read_u8(data: &[u8], offset: usize) -> Result<u8> {
    data.get(offset).copied().ok_or(NuccError::Truncated)
}

fn read_u16le(data: &[u8], offset: usize) -> Result<u16> {
    let bytes = data.get(offset..offset + 2).ok_or(NuccError::Truncated)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32le(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data.get(offset..offset + 4).ok_or(NuccError::Truncated)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u32be(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data.get(offset..offset + 4).ok_or(NuccError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn add(left: usize, right: usize) -> Result<usize> {
    left.checked_add(right).ok_or(NuccError::Overflow)
}

fn align(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_bank() {
        let data = minimal_bank();
        let bank = Nus3Bank::parse(&data).unwrap();

        assert_eq!(bank.file_size, data.len());
        assert_eq!(bank.chunks.len(), 2);
        assert_eq!(bank.tones.len(), 1);
        assert_eq!(bank.tones[0].name, "a");
        assert_eq!(bank.tones[0].codec, Nus3Codec::Bnsf);
        assert_eq!(bank.tone_stream(0).unwrap(), b"BNSF");
    }

    #[test]
    fn rejects_non_nus3_data() {
        let error = Nus3Bank::parse(&[0; 0x18]).unwrap_err();
        assert!(matches!(error, NuccError::InvalidHeader));
    }

    fn minimal_bank() -> Vec<u8> {
        let mut data = vec![0u8; 0x64];

        data[0x00..0x04].copy_from_slice(b"NUS3");
        data[0x04..0x08].copy_from_slice(&0x64u32.to_le_bytes());
        data[0x08..0x0c].copy_from_slice(b"BANK");
        data[0x0c..0x10].copy_from_slice(b"TOC ");
        data[0x10..0x14].copy_from_slice(&0x14u32.to_le_bytes());
        data[0x14..0x18].copy_from_slice(&2u32.to_le_bytes());
        data[0x18..0x1c].copy_from_slice(b"TONE");
        data[0x1c..0x20].copy_from_slice(&0x28u32.to_le_bytes());
        data[0x20..0x24].copy_from_slice(b"PACK");
        data[0x24..0x28].copy_from_slice(&4u32.to_le_bytes());

        data[0x28..0x2c].copy_from_slice(b"TONE");
        data[0x2c..0x30].copy_from_slice(&0x28u32.to_le_bytes());
        data[0x30..0x34].copy_from_slice(&1u32.to_le_bytes());
        data[0x34..0x38].copy_from_slice(&0x0cu32.to_le_bytes());
        data[0x38..0x3c].copy_from_slice(&0x1cu32.to_le_bytes());
        data[0x40..0x42].copy_from_slice(&0xffffu16.to_le_bytes());
        data[0x44] = 2;
        data[0x45..0x47].copy_from_slice(b"a\0");
        data[0x4c..0x50].copy_from_slice(&8u32.to_le_bytes());
        data[0x54..0x58].copy_from_slice(&4u32.to_le_bytes());

        data[0x58..0x5c].copy_from_slice(b"PACK");
        data[0x5c..0x60].copy_from_slice(&4u32.to_le_bytes());
        data[0x60..0x64].copy_from_slice(b"BNSF");

        data
    }
}
