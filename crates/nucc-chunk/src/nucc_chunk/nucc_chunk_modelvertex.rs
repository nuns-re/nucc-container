use std::io::Write;

use binrw::{binrw, io::Cursor, BinRead, BinWrite};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkModelVertex {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub data_size: u32,
    #[br(count = data_size)]
    pub data: Vec<u8>,

    #[br(calc = None)]
    #[bw(ignore)]
    pub reference: Option<NuccChunkReference>,
    #[br(calc = None)]
    #[bw(ignore)]
    pub raw_data: Option<Vec<u8>>,
}

impl NuccChunk for NuccChunkModelVertex {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkModelVertex
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let model = NuccChunkModelVertex {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(model)));
        }

        let mut cursor = Cursor::new(input);
        match Self::read_be_args(&mut cursor, (version,)) {
            Ok(model) => Ok((input.to_vec(), Box::new(model))),
            Err(_) => {
                let model = NuccChunkModelVertex {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(model)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let model = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = model.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &model.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        model.write_be_args(output, (version,))?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

#[cfg(feature = "nu")]
impl NuccChunkModelVertex {
    pub fn nud(&self) -> nucc_core::Result<nu::model::Nud> {
        nu::model::Nud::parse(&self.data)
            .map_err(|error| nucc_core::NuccError::InvalidChunk(format!("NUD: {error}")))
    }
}
