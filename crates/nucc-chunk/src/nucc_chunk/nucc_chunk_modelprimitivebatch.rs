use std::io::{Read, Seek, Write};

use binrw::{io::Cursor, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NuccModelPrimitiveBatchEntry {
    pub coord_id: u16,
    pub vertex_num: u16,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkModelPrimitiveBatch {
    pub version: u16,
    pub coord_index: u32,
    pub unknown_index: u32,
    pub primitive_vertex_index: u32,
    pub batch_count: u16,
    pub flags: u16,
    pub field3c: u16,
    pub field3e: u16,
    pub field24: u16,
    pub field40: u32,
    pub field44: u8,
    pub field48: u8,
    pub reserved: u32,
    pub shared: [u32; 3],
    pub optional: Option<[u32; 2]>,
    pub batches: Vec<NuccModelPrimitiveBatchEntry>,
    pub trailing: Vec<u8>,
    pub reference: Option<NuccChunkReference>,
    pub raw_data: Option<Vec<u8>>,
}

impl BinRead for NuccChunkModelPrimitiveBatch {
    type Args = (u16,);

    fn read_options<R: Read + Seek>(
        _reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkModelPrimitiveBatch uses read_boxed",
        )
        .into())
    }
}

impl BinWrite for NuccChunkModelPrimitiveBatch {
    type Args = (u16,);

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkModelPrimitiveBatch uses write_boxed",
        )
        .into())
    }
}

impl NuccChunk for NuccChunkModelPrimitiveBatch {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkModelPrimitiveBatch
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let batch = NuccChunkModelPrimitiveBatch {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(batch)));
        }

        match decode_model_primitive_batch(input, version) {
            Ok(batch) => Ok((input.to_vec(), Box::new(batch))),
            Err(_) => {
                let batch = NuccChunkModelPrimitiveBatch {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(batch)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let batch = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = batch.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &batch.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        output.write_all(&encode_model_primitive_batch(&batch, version)?)?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

fn decode_model_primitive_batch(
    input: &[u8],
    version: u16,
) -> Result<NuccChunkModelPrimitiveBatch> {
    let mut reader = Reader::new(input);
    let coord_index = reader.u32()?;
    let unknown_index = reader.u32()?;
    let primitive_vertex_index = reader.u32()?;
    let wide = version > 0x76;
    let (batch_count, flags, field3c, field3e, field24, field40, field44, field48, reserved) =
        if wide {
            let reserved = reader.u32()?;
            let batch_count = reader.u16()?;
            let flags = reader.u16()?;
            let field3c = reader.u16()?;
            let field3e = reader.u16()?;
            let field24 = reader.u16()?;
            let packed = reader.u16()?;
            (
                batch_count,
                flags,
                field3c,
                field3e,
                field24,
                reader.u32()?,
                packed as u8,
                (packed >> 8) as u8,
                reserved,
            )
        } else {
            let batch_count = reader.u16()?;
            let flags = reader.u16()?;
            let field3c = reader.u16()?;
            let field3e = reader.u16()?;
            let reserved = reader.u32()?;
            (
                batch_count,
                flags,
                field3c,
                field3e,
                0,
                reader.u32()?,
                0,
                0,
                reserved,
            )
        };

    let shared = [reader.u32()?, reader.u32()?, reader.u32()?];
    let optional = if flags == 1 {
        Some([reader.u32()?, reader.u32()?])
    } else {
        None
    };
    let mut batches = Vec::with_capacity(batch_count as usize);
    for _ in 0..batch_count {
        batches.push(NuccModelPrimitiveBatchEntry {
            coord_id: reader.u16()?,
            vertex_num: reader.u16()?,
            index: reader.u32()?,
        });
    }

    Ok(NuccChunkModelPrimitiveBatch {
        version,
        coord_index,
        unknown_index,
        primitive_vertex_index,
        batch_count,
        flags,
        field3c,
        field3e,
        field24,
        field40,
        field44,
        field48,
        reserved,
        shared,
        optional,
        batches,
        trailing: reader.remaining().to_vec(),
        reference: None,
        raw_data: None,
    })
}

fn encode_model_primitive_batch(
    batch: &NuccChunkModelPrimitiveBatch,
    version: u16,
) -> Result<Vec<u8>> {
    let wide = version > 0x76;
    let mut output = Vec::new();
    output.extend_from_slice(&batch.coord_index.to_be_bytes());
    output.extend_from_slice(&batch.unknown_index.to_be_bytes());
    output.extend_from_slice(&batch.primitive_vertex_index.to_be_bytes());
    if wide {
        output.extend_from_slice(&batch.reserved.to_be_bytes());
        output.extend_from_slice(&batch.batch_count.to_be_bytes());
        output.extend_from_slice(&batch.flags.to_be_bytes());
        output.extend_from_slice(&batch.field3c.to_be_bytes());
        output.extend_from_slice(&batch.field3e.to_be_bytes());
        output.extend_from_slice(&batch.field24.to_be_bytes());
        output.extend_from_slice(&[batch.field48, batch.field44]);
        output.extend_from_slice(&batch.field40.to_be_bytes());
    } else {
        output.extend_from_slice(&batch.batch_count.to_be_bytes());
        output.extend_from_slice(&batch.flags.to_be_bytes());
        output.extend_from_slice(&batch.field3c.to_be_bytes());
        output.extend_from_slice(&batch.field3e.to_be_bytes());
        output.extend_from_slice(&batch.reserved.to_be_bytes());
        output.extend_from_slice(&batch.field40.to_be_bytes());
    }
    for value in batch.shared {
        output.extend_from_slice(&value.to_be_bytes());
    }
    if batch.flags == 1 {
        let optional = batch.optional.ok_or_else(|| {
            NuccError::InvalidChunk("ModelPrimitiveBatch optional values missing".into())
        })?;
        for value in optional {
            output.extend_from_slice(&value.to_be_bytes());
        }
    }
    for entry in &batch.batches {
        output.extend_from_slice(&entry.coord_id.to_be_bytes());
        output.extend_from_slice(&entry.vertex_num.to_be_bytes());
        output.extend_from_slice(&entry.index.to_be_bytes());
    }
    output.extend_from_slice(&batch.trailing);
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

    fn remaining(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }
}
