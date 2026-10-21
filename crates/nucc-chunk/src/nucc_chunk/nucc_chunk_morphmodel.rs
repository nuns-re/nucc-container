use std::io::Write;

use binrw::{binrw, io::Cursor, BinRead, BinWrite};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccMorphModelTarget {
    pub vertex_target_index: u32,
    pub weight: f32,
}

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkMorphModel {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub target_num: u16,
    pub reserved: u16,
    pub model_base_index: u32,
    pub reserved2: u32,
    #[br(count = target_num)]
    pub targets: Vec<NuccMorphModelTarget>,

    #[br(calc = None)]
    #[bw(ignore)]
    pub reference: Option<NuccChunkReference>,
    #[br(calc = None)]
    #[bw(ignore)]
    pub raw_data: Option<Vec<u8>>,
}

impl NuccChunk for NuccChunkMorphModel {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkMorphModel
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let morph = NuccChunkMorphModel {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(morph)));
        }

        let mut cursor = Cursor::new(input);
        match Self::read_be_args(&mut cursor, (version,)) {
            Ok(morph) => Ok((input.to_vec(), Box::new(morph))),
            Err(_) => {
                let morph = NuccChunkMorphModel {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(morph)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let morph = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = morph.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &morph.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        morph.write_be_args(output, (version,))?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}
