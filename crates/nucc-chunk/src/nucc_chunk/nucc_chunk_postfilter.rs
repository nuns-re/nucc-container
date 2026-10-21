use std::io::{Read, Seek, Write};

use binrw::{io::Cursor, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkPostFilter {
    pub version: u16,
    pub filter_type: u16,
    pub params: Vec<f32>,
    pub reference: Option<NuccChunkReference>,
    pub raw_data: Option<Vec<u8>>,
}

impl BinRead for NuccChunkPostFilter {
    type Args = (u16,);

    fn read_options<R: Read + Seek>(
        _reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkPostFilter uses read_boxed",
        )
        .into())
    }
}

impl BinWrite for NuccChunkPostFilter {
    type Args = (u16,);

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkPostFilter uses write_boxed",
        )
        .into())
    }
}

impl NuccChunk for NuccChunkPostFilter {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkPostFilter
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let post_filter = NuccChunkPostFilter {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(post_filter)));
        }

        match decode_post_filter(input, version) {
            Ok(post_filter) => Ok((input.to_vec(), Box::new(post_filter))),
            Err(_) => {
                let post_filter = NuccChunkPostFilter {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(post_filter)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let post_filter = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = post_filter.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &post_filter.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        output.write_all(&encode_post_filter(&post_filter, version)?)?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

fn decode_post_filter(input: &[u8], version: u16) -> Result<NuccChunkPostFilter> {
    if input.len() < 4 {
        return Err(NuccError::Truncated);
    }
    let filter_type = u16::from_be_bytes([input[0], input[1]]);
    let count = u16::from_be_bytes([input[2], input[3]]) as usize;
    let end = 4usize
        .checked_add(count.checked_mul(4).ok_or(NuccError::Overflow)?)
        .ok_or(NuccError::Overflow)?;
    let data = input.get(4..end).ok_or(NuccError::Truncated)?;
    let mut params = Vec::with_capacity(count);
    for chunk in data.as_chunks::<4>().0 {
        params.push(f32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(NuccChunkPostFilter {
        version,
        filter_type,
        params,
        reference: None,
        raw_data: None,
    })
}

fn encode_post_filter(post_filter: &NuccChunkPostFilter, _version: u16) -> Result<Vec<u8>> {
    let count = u16::try_from(post_filter.params.len())
        .map_err(|_| NuccError::InvalidChunk("PostFilter parameter count overflow".into()))?;
    let mut output = Vec::with_capacity(4 + post_filter.params.len() * 4);
    output.extend_from_slice(&post_filter.filter_type.to_be_bytes());
    output.extend_from_slice(&count.to_be_bytes());
    for value in &post_filter.params {
        output.extend_from_slice(&value.to_bits().to_be_bytes());
    }
    Ok(output)
}
