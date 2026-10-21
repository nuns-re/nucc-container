use std::io::Write;

use binrw::{binrw, until_eof, BinRead, BinWrite};

use super::{NuccChunk, NuccChunkCoord, NuccChunkReference, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkTexture {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    pub field00: u16,
    pub width: u16,
    pub height: u16,
    pub field06: u16,
    #[bw(calc = nut_data.len() as u32)]
    #[br(assert(size <= 512 * 1024 * 1024, "NUT payload exceeds limit"))]
    pub size: u32,
    #[br(count = size)]
    pub nut_data: Vec<u8>,
    #[br(parse_with = until_eof)]
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

impl NuccChunk for NuccChunkTexture {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkTexture
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if input.len() == 8 {
            let reference = NuccChunkReference {
                indices: [read_u32be(input, 0)?, read_u32be(input, 4)?],
            };
            let texture = NuccChunkTexture {
                version,
                field00: 0,
                width: 0,
                height: 0,
                field06: 0,
                nut_data: Vec::new(),
                trailing_data: Vec::new(),
                reference: Some(reference),
                coord: None,
                raw_data: None,
            };
            return Ok((input.to_vec(), Box::new(texture)));
        }

        if input.len() == 42 {
            if let Ok((_, chunk)) = NuccChunkCoord::read_boxed(input, version) {
                if let Ok(coord) = chunk.downcast::<NuccChunkCoord>() {
                    let texture = NuccChunkTexture {
                        version,
                        field00: 0,
                        width: 0,
                        height: 0,
                        field06: 0,
                        nut_data: Vec::new(),
                        trailing_data: Vec::new(),
                        reference: None,
                        coord: Some(*coord),
                        raw_data: None,
                    };
                    return Ok((input.to_vec(), Box::new(texture)));
                }
            }
        }

        let mut cursor = binrw::io::Cursor::new(input);
        match Self::read_be_args(&mut cursor, (version,)) {
            Ok(texture) => Ok((input.to_vec(), Box::new(texture))),
            Err(_) => {
                let texture = NuccChunkTexture {
                    version,
                    field00: 0,
                    width: 0,
                    height: 0,
                    field06: 0,
                    nut_data: Vec::new(),
                    trailing_data: Vec::new(),
                    reference: None,
                    coord: None,
                    raw_data: Some(input.to_vec()),
                };
                Ok((input.to_vec(), Box::new(texture)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut binrw::io::Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let texture = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = texture.reference {
            for index in reference.indices {
                output.write_all(&index.to_be_bytes())?;
            }
            return Ok(());
        }
        if let Some(raw) = &texture.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        if let Some(coord) = &texture.coord {
            NuccChunkCoord::write_boxed(Box::new(coord.clone()), output, version)?;
            return Ok(());
        }
        texture.write_be_args(output, (version,))?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

fn read_u32be(data: &[u8], at: usize) -> nucc_core::Result<u32> {
    let bytes = data
        .get(at..at + 4)
        .ok_or(nucc_core::NuccError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(feature = "nu")]
impl NuccChunkTexture {
    pub fn nut(&self) -> nucc_core::Result<nu::texture::Nut> {
        nu::texture::Nut::parse(&self.nut_data)
            .map_err(|error| nucc_core::NuccError::InvalidChunk(format!("NUT: {error}")))
    }
}
