use std::io::Write;

use binrw::{binrw, io::Cursor, BinRead, BinWrite};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccLightSetCategory {
    pub index: u8,
    pub flags: u8,
    pub name_offset: u16,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccLightSetCategoryGroup {
    pub index: u8,
    pub flags: u8,
    pub name_offset: u16,
    pub category: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccLightSetMode {
    pub index: u8,
    pub flags: u8,
    pub name_offset: u16,
}

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkLightSet {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub category_count: u8,
    pub category_group_count: u8,
    pub mode_count: u8,
    pub reserved: u8,
    pub string_table_size: u16,
    pub reserved2: u16,
    #[br(count = category_count)]
    pub categories: Vec<NuccLightSetCategory>,
    #[br(count = category_group_count)]
    pub category_groups: Vec<NuccLightSetCategoryGroup>,
    #[br(count = mode_count)]
    pub modes: Vec<NuccLightSetMode>,
    #[br(count = string_table_size)]
    pub strings: Vec<u8>,

    #[br(calc = None)]
    #[bw(ignore)]
    pub reference: Option<NuccChunkReference>,
    #[br(calc = None)]
    #[bw(ignore)]
    pub raw_data: Option<Vec<u8>>,
}

impl NuccChunkLightSet {
    pub fn string_at(&self, offset: u16) -> Option<&str> {
        let tail = self.strings.get(offset as usize..)?;
        let end = tail
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(tail.len());
        std::str::from_utf8(&tail[..end]).ok()
    }
}

impl NuccChunk for NuccChunkLightSet {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkLightSet
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let light_set = NuccChunkLightSet {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(light_set)));
        }

        let mut cursor = Cursor::new(input);
        match Self::read_be_args(&mut cursor, (version,)) {
            Ok(light_set) => Ok((input.to_vec(), Box::new(light_set))),
            Err(_) => {
                let light_set = NuccChunkLightSet {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(light_set)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let light_set = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = light_set.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &light_set.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        light_set.write_be_args(output, (version,))?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}
