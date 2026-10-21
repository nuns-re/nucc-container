use std::io::{Read, Seek, Write};

use binrw::{io::Cursor, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkPrimitiveVertex {
    pub version: u16,
    pub primitive_type: u32,
    pub vertex_format: u32,
    pub vertex_stride: u32,
    pub vertex_count: u32,
    pub data: Vec<u8>,
    pub reference: Option<NuccChunkReference>,
    pub raw_data: Option<Vec<u8>>,
}

impl BinRead for NuccChunkPrimitiveVertex {
    type Args = (u16,);

    fn read_options<R: Read + Seek>(
        _reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkPrimitiveVertex uses read_boxed",
        )
        .into())
    }
}

impl BinWrite for NuccChunkPrimitiveVertex {
    type Args = (u16,);

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkPrimitiveVertex uses write_boxed",
        )
        .into())
    }
}

impl NuccChunk for NuccChunkPrimitiveVertex {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkPrimitiveVertex
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let vertex = NuccChunkPrimitiveVertex {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(vertex)));
        }

        match decode_primitive_vertex(input, version) {
            Ok(vertex) => Ok((input.to_vec(), Box::new(vertex))),
            Err(_) => {
                let vertex = NuccChunkPrimitiveVertex {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(vertex)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let vertex = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = vertex.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &vertex.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        output.write_all(&encode_primitive_vertex(&vertex, version)?)?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

fn decode_primitive_vertex(input: &[u8], version: u16) -> Result<NuccChunkPrimitiveVertex> {
    if input.len() < 16 {
        return Err(NuccError::Truncated);
    }
    let primitive_type = read_u32(input, 0)?;
    let vertex_format = read_u32(input, 4)?;
    let vertex_stride = read_u32(input, 8)?;
    let vertex_count = read_u32(input, 12)?;
    let size = (vertex_count as usize)
        .checked_mul(vertex_stride as usize)
        .ok_or(NuccError::Overflow)?;
    let end = 16usize.checked_add(size).ok_or(NuccError::Overflow)?;
    let data = input.get(16..end).ok_or(NuccError::Truncated)?.to_vec();
    Ok(NuccChunkPrimitiveVertex {
        version,
        primitive_type,
        vertex_format,
        vertex_stride,
        vertex_count,
        data,
        reference: None,
        raw_data: None,
    })
}

fn encode_primitive_vertex(vertex: &NuccChunkPrimitiveVertex, _version: u16) -> Result<Vec<u8>> {
    let expected = (vertex.vertex_count as usize)
        .checked_mul(vertex.vertex_stride as usize)
        .ok_or(NuccError::Overflow)?;
    if expected != vertex.data.len() {
        return Err(NuccError::InvalidChunk(
            "PrimitiveVertex data does not match count and stride".into(),
        ));
    }
    let mut output = Vec::with_capacity(16 + vertex.data.len());
    output.extend_from_slice(&vertex.primitive_type.to_be_bytes());
    output.extend_from_slice(&vertex.vertex_format.to_be_bytes());
    output.extend_from_slice(&vertex.vertex_stride.to_be_bytes());
    output.extend_from_slice(&vertex.vertex_count.to_be_bytes());
    output.extend_from_slice(&vertex.data);
    Ok(output)
}

fn read_u32(data: &[u8], at: usize) -> Result<u32> {
    let bytes = data.get(at..at + 4).ok_or(NuccError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
