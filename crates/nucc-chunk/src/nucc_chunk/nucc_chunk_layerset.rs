use std::io::Write;

use binrw::{binrw, io::Cursor, BinRead, BinWrite};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccLayerSetLayer {
    pub index: u8,
    pub flags: u8,
    pub name_offset: u16,
    pub layer_offset: u16,
    pub priority: u16,
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccLayerSetSelector {
    pub index: u8,
    pub flags: u8,
    pub name_offset: u16,
    pub layer_offset: u16,
    pub selector_offset: u16,
}

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkLayerSet {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub layer_count: u8,
    pub selector_count: u8,
    pub string_table_size: u16,
    pub layer_selector_offset: u16,
    #[br(count = layer_count)]
    pub layers: Vec<NuccLayerSetLayer>,
    #[br(count = selector_count)]
    pub selectors: Vec<NuccLayerSetSelector>,
    #[br(count = string_table_size)]
    pub strings: Vec<u8>,

    #[br(calc = None)]
    #[bw(ignore)]
    pub reference: Option<NuccChunkReference>,
    #[br(calc = None)]
    #[bw(ignore)]
    pub raw_data: Option<Vec<u8>>,
}

impl NuccChunkLayerSet {
    pub fn string_at(&self, offset: u16) -> Option<&str> {
        let tail = self.strings.get(offset as usize..)?;
        let end = tail
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(tail.len());
        std::str::from_utf8(&tail[..end]).ok()
    }
}

impl NuccChunk for NuccChunkLayerSet {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkLayerSet
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let layer_set = NuccChunkLayerSet {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(layer_set)));
        }

        let mut cursor = Cursor::new(input);
        match Self::read_be_args(&mut cursor, (version,)) {
            Ok(layer_set) => Ok((input.to_vec(), Box::new(layer_set))),
            Err(_) => {
                let layer_set = NuccChunkLayerSet {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(layer_set)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let layer_set = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = layer_set.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &layer_set.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        layer_set.write_be_args(output, (version,))?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}
