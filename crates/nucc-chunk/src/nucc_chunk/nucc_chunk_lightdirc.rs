use std::io::Write;

use binrw::{binrw, io::Cursor, BinRead, BinWrite};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkLightDirc {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub index: u32,
    pub reserved: [u8; 12],
    pub color: [f32; 3],
    pub intensity: f32,
    pub reserved2: [u8; 16],
    pub direction: [f32; 4],

    #[br(calc = None)]
    #[bw(ignore)]
    pub reference: Option<NuccChunkReference>,
    #[br(calc = None)]
    #[bw(ignore)]
    pub raw_data: Option<Vec<u8>>,
}

impl NuccChunk for NuccChunkLightDirc {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkLightDirc
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let light = NuccChunkLightDirc {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(light)));
        }

        let mut cursor = Cursor::new(input);
        match Self::read_be_args(&mut cursor, (version,)) {
            Ok(light) => Ok((input.to_vec(), Box::new(light))),
            Err(_) => {
                let light = NuccChunkLightDirc {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(light)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let light = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = light.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &light.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        light.write_be_args(output, (version,))?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}
