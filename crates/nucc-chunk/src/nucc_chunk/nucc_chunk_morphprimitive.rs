use std::io::{Read, Seek, Write};

use binrw::{io::Cursor, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccMorphPrimitiveTarget {
    pub primitive_vertex_target_index: u32,
    pub weight: f32,
    pub reserved: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkMorphPrimitive {
    pub version: u16,
    pub target_num: u16,
    pub reserved: u16,
    pub attribute: u32,
    pub model_primitive_base_index: u32,
    pub reserved2: u32,
    pub targets: Vec<NuccMorphPrimitiveTarget>,
    pub reference: Option<NuccChunkReference>,
    pub raw_data: Option<Vec<u8>>,
}

impl BinRead for NuccChunkMorphPrimitive {
    type Args = (u16,);

    fn read_options<R: Read + Seek>(
        _reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkMorphPrimitive uses read_boxed",
        )
        .into())
    }
}

impl BinWrite for NuccChunkMorphPrimitive {
    type Args = (u16,);

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkMorphPrimitive uses write_boxed",
        )
        .into())
    }
}

impl NuccChunk for NuccChunkMorphPrimitive {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkMorphPrimitive
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let morph = NuccChunkMorphPrimitive {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(morph)));
        }

        match decode_morph_primitive(input, version) {
            Ok(morph) => Ok((input.to_vec(), Box::new(morph))),
            Err(_) => {
                let morph = NuccChunkMorphPrimitive {
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
        output.write_all(&encode_morph_primitive(&morph, version)?)?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

fn decode_morph_primitive(input: &[u8], version: u16) -> Result<NuccChunkMorphPrimitive> {
    let mut reader = Reader::new(input);
    let target_num = reader.u16()?;
    let reserved = reader.u16()?;
    let attribute = reader.u32()?;
    let model_primitive_base_index = reader.u32()?;
    let reserved2 = reader.u32()?;
    let mut targets = Vec::with_capacity(target_num as usize);
    for _ in 0..target_num {
        targets.push(NuccMorphPrimitiveTarget {
            primitive_vertex_target_index: reader.u32()?,
            weight: reader.f32()?,
            reserved: reader.u32()?,
        });
    }
    Ok(NuccChunkMorphPrimitive {
        version,
        target_num,
        reserved,
        attribute,
        model_primitive_base_index,
        reserved2,
        targets,
        reference: None,
        raw_data: None,
    })
}

fn encode_morph_primitive(morph: &NuccChunkMorphPrimitive, _version: u16) -> Result<Vec<u8>> {
    if morph.target_num as usize != morph.targets.len() {
        return Err(NuccError::InvalidChunk(
            "MorphPrimitive target count does not match".into(),
        ));
    }
    let mut output = Vec::new();
    output.extend_from_slice(&morph.target_num.to_be_bytes());
    output.extend_from_slice(&morph.reserved.to_be_bytes());
    output.extend_from_slice(&morph.attribute.to_be_bytes());
    output.extend_from_slice(&morph.model_primitive_base_index.to_be_bytes());
    output.extend_from_slice(&morph.reserved2.to_be_bytes());
    for target in &morph.targets {
        output.extend_from_slice(&target.primitive_vertex_target_index.to_be_bytes());
        output.extend_from_slice(&target.weight.to_bits().to_be_bytes());
        output.extend_from_slice(&target.reserved.to_be_bytes());
    }
    Ok(output)
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn take(&mut self, size: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(size).ok_or(NuccError::Overflow)?;
        let bytes = self
            .data
            .get(self.offset..end)
            .ok_or(NuccError::Truncated)?;
        self.offset = end;
        Ok(bytes)
    }

    fn u16(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn f32(&mut self) -> Result<f32> {
        let bytes = self.take(4)?;
        Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}
