use nucc_core::{NuccError, Result};

const HEADER_SIZE: usize = 0x10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwbHeader {
    pub version: u8,
    pub offset_size: u8,
    pub waveid_alignment: u16,
    pub offset_alignment: u16,
    pub subkey: u16,
    pub entry_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwbCodec {
    Adx,
    Hca,
    Vag,
    Riff,
    Cwav,
    Dsp,
    Cwac,
    Opus,
    M4a,
    Idsp,
    Bnsf,
    Ivag,
    Unknown([u8; 4]),
    Empty,
}

impl AwbCodec {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Adx => "adx",
            Self::Hca => "hca",
            Self::Vag => "vag",
            Self::Riff => "wav",
            Self::Cwav => "bcwav",
            Self::Dsp | Self::Cwac => "dsp",
            Self::Opus => "opus",
            Self::M4a => "m4a",
            Self::Idsp => "idsp",
            Self::Bnsf => "bnsf",
            Self::Ivag => "ivag",
            Self::Unknown(_) | Self::Empty => "bin",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwbEntry {
    pub index: usize,
    pub id: u16,
    pub offset: usize,
    pub size: usize,
    pub codec: AwbCodec,
}

#[derive(Debug, Clone)]
pub struct Awb {
    data: Vec<u8>,
    pub header: AwbHeader,
    pub entries: Vec<AwbEntry>,
}

impl Awb {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(NuccError::Truncated);
        }
        if &data[0..4] != b"AFS2" {
            return Err(NuccError::InvalidHeader);
        }

        let version = data[4];
        let offset_size = data[5];
        let waveid_alignment = read_u16le(data, 0x06)?;
        let entry_count = read_i32le(data, 0x08)?;
        let offset_alignment = read_u16le(data, 0x0c)?;
        let subkey = read_u16le(data, 0x0e)?;

        if entry_count <= 0 {
            return Err(NuccError::InvalidHeader);
        }
        if !matches!(offset_size, 2 | 4) {
            return Err(NuccError::Unsupported(format!(
                "AWB offset size {offset_size}"
            )));
        }
        if waveid_alignment == 0 || offset_alignment == 0 {
            return Err(NuccError::InvalidHeader);
        }

        let entry_count = entry_count as usize;
        let id_table_size = entry_count
            .checked_mul(waveid_alignment as usize)
            .ok_or(NuccError::Overflow)?;
        let offset_table = add(HEADER_SIZE, id_table_size)?;
        let offset_count = entry_count.checked_add(1).ok_or(NuccError::Overflow)?;
        let offset_table_size = offset_count
            .checked_mul(offset_size as usize)
            .ok_or(NuccError::Overflow)?;
        let offset_table_end = add(offset_table, offset_table_size)?;
        if offset_table_end > data.len() {
            return Err(NuccError::InvalidIndex(
                "AWB offset table exceeds file".into(),
            ));
        }

        let mut offsets = Vec::with_capacity(offset_count);
        for index in 0..offset_count {
            let position = add(offset_table, index * offset_size as usize)?;
            let value = match offset_size {
                2 => read_u16le(data, position)? as usize,
                _ => read_u32le(data, position)? as usize,
            };
            let value = if index < entry_count || value < data.len() {
                align_up(value, offset_alignment as usize)?
            } else {
                value
            };
            offsets.push(value);
        }

        let mut entries = Vec::with_capacity(entry_count);
        for index in 0..entry_count {
            let id_position = add(HEADER_SIZE, index * waveid_alignment as usize)?;
            let id = read_u16le(data, id_position)?;
            let offset = offsets[index];
            let end = offsets[index + 1];

            if offset > data.len() || end > data.len() || end < offset {
                return Err(NuccError::InvalidIndex(format!(
                    "AWB entry {index} has invalid range"
                )));
            }

            let stream = &data[offset..end];
            entries.push(AwbEntry {
                index,
                id,
                offset,
                size: end - offset,
                codec: detect_codec(stream),
            });
        }

        Ok(Self {
            data: data.to_vec(),
            header: AwbHeader {
                version,
                offset_size,
                waveid_alignment,
                offset_alignment,
                subkey,
                entry_count,
            },
            entries,
        })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn entry(&self, index: usize) -> Option<&AwbEntry> {
        self.entries.get(index)
    }

    pub fn entry_by_id(&self, id: u16) -> Option<&AwbEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn stream(&self, index: usize) -> Option<&[u8]> {
        let entry = self.entries.get(index)?;
        self.entry_stream(entry)
    }

    pub fn entry_stream(&self, entry: &AwbEntry) -> Option<&[u8]> {
        let end = entry.offset.checked_add(entry.size)?;
        self.data.get(entry.offset..end)
    }
}

fn detect_codec(data: &[u8]) -> AwbCodec {
    if data.is_empty() {
        return AwbCodec::Empty;
    }
    if data.len() >= 2 && u16::from_be_bytes([data[0], data[1]]) == 0x8000 {
        return AwbCodec::Adx;
    }
    if data.len() < 4 {
        return AwbCodec::Unknown([0; 4]);
    }

    let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    if magic & 0x7f7f_7f7f == 0x4843_4100 {
        return AwbCodec::Hca;
    }

    match magic {
        0x5641_4770 => return AwbCodec::Vag,
        0x5249_4646 => return AwbCodec::Riff,
        0x4357_4156 => return AwbCodec::Cwav,
        0x4357_4143 => return AwbCodec::Cwac,
        0x4944_5350 => return AwbCodec::Idsp,
        0x424e_5346 => return AwbCodec::Bnsf,
        0x4956_4147 => return AwbCodec::Ivag,
        0x4f50_5553 => return AwbCodec::Opus,
        0x0100_0080 => return AwbCodec::Opus,
        _ => {}
    }

    if data.len() >= 8 && magic == 0x0000_0018 && &data[4..8] == b"ftyp" {
        return AwbCodec::M4a;
    }

    if is_dsp(data) {
        return AwbCodec::Dsp;
    }

    AwbCodec::Unknown(magic.to_be_bytes())
}

fn is_dsp(data: &[u8]) -> bool {
    if data.len() < 0x54 {
        return false;
    }
    let sample_rate = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let format = u16::from_be_bytes([data[0x0e], data[0x0f]]);
    let loop_flag = u32::from_be_bytes([data[0x18], data[0x19], data[0x1a], data[0x1b]]);
    let last = u32::from_be_bytes([data[0x50], data[0x51], data[0x52], data[0x53]]);
    (8000..=48000).contains(&sample_rate) && format == 0 && loop_flag == 2 && last == 0
}

fn read_i32le(data: &[u8], offset: usize) -> Result<i32> {
    let bytes = data.get(offset..offset + 4).ok_or(NuccError::Truncated)?;
    Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u16le(data: &[u8], offset: usize) -> Result<u16> {
    let bytes = data.get(offset..offset + 2).ok_or(NuccError::Truncated)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32le(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data.get(offset..offset + 4).ok_or(NuccError::Truncated)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn add(left: usize, right: usize) -> Result<usize> {
    left.checked_add(right).ok_or(NuccError::Overflow)
}

fn align_up(value: usize, alignment: usize) -> Result<usize> {
    let remainder = value % alignment;
    if remainder == 0 {
        Ok(value)
    } else {
        add(value, alignment - remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entries() {
        let data = minimal_awb();
        let awb = Awb::parse(&data).unwrap();

        assert_eq!(awb.header.entry_count, 2);
        assert_eq!(awb.entries.len(), 2);
        assert_eq!(awb.entries[0].id, 0);
        assert_eq!(awb.entries[0].codec, AwbCodec::Hca);
        assert_eq!(awb.entries[1].id, 1);
        assert_eq!(awb.entries[1].codec, AwbCodec::Vag);
        assert_eq!(awb.stream(0).unwrap().len(), 0x20);
    }

    #[test]
    fn rejects_non_awb_data() {
        let error = Awb::parse(&[0; 0x10]).unwrap_err();
        assert!(matches!(error, NuccError::InvalidHeader));
    }

    fn minimal_awb() -> Vec<u8> {
        let mut data = vec![0u8; 0x60];

        data[0x00..0x04].copy_from_slice(b"AFS2");
        data[0x04] = 1;
        data[0x05] = 4;
        data[0x06..0x08].copy_from_slice(&2u16.to_le_bytes());
        data[0x08..0x0c].copy_from_slice(&2i32.to_le_bytes());
        data[0x0c..0x0e].copy_from_slice(&0x20u16.to_le_bytes());

        data[0x10..0x12].copy_from_slice(&0u16.to_le_bytes());
        data[0x12..0x14].copy_from_slice(&1u16.to_le_bytes());
        data[0x14..0x18].copy_from_slice(&0x20u32.to_le_bytes());
        data[0x18..0x1c].copy_from_slice(&0x40u32.to_le_bytes());
        data[0x1c..0x20].copy_from_slice(&0x60u32.to_le_bytes());

        data[0x20..0x24].copy_from_slice(b"HCA\0");
        data[0x40..0x44].copy_from_slice(b"VAGp");

        data
    }
}
