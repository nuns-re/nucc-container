use std::io::{Read, Seek, Write};

use binrw::{io::Cursor, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use nucc_core::{NuccError, Result};

use super::{NuccChunk, NuccChunkReference, NuccChunkType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuccModelHitObject {
    pub poly_count: u32,
    /// Native `a`: a 32-bit attribute word. Its low 29 bits are OR'ed with the
    /// runtime orientation class; the top three wire bits are discarded.
    pub attribute_a: u32,
    /// Native `b`: stored raw at runtime polygon `+0x50`. Semantics are not
    /// established; the player line/sphere predicate does not read it.
    pub attribute_b: u32,
    /// Fourth word of the wide (`version > 0x79`) group header. `Parse` reads
    /// and discards it, so it is reserved rather than an attribute.
    pub reserved: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NuccModelHitTriangle {
    pub v: [[f32; 3]; 3],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NuccChunkModelHit {
    pub version: u16,
    pub objects: Vec<NuccModelHitObject>,
    pub triangles: Vec<NuccModelHitTriangle>,
    pub trailing: Vec<u8>,
    pub reference: Option<NuccChunkReference>,
    pub raw_data: Option<Vec<u8>>,
}

impl BinRead for NuccChunkModelHit {
    type Args = (u16,);

    fn read_options<R: Read + Seek>(
        _reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkModelHit uses read_boxed",
        )
        .into())
    }
}

impl BinWrite for NuccChunkModelHit {
    type Args = (u16,);

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "NuccChunkModelHit uses write_boxed",
        )
        .into())
    }
}

impl NuccChunk for NuccChunkModelHit {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkModelHit
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn read_boxed(input: &[u8], version: u16) -> super::NuccChunkReadResult {
        if let Some(reference) = NuccChunkReference::parse(input) {
            let hit = NuccChunkModelHit {
                version,
                reference: Some(reference),
                ..Default::default()
            };
            return Ok((input.to_vec(), Box::new(hit)));
        }

        match decode_model_hit(input, version) {
            Ok(hit) => Ok((input.to_vec(), Box::new(hit))),
            Err(_) => {
                let hit = NuccChunkModelHit {
                    version,
                    raw_data: Some(input.to_vec()),
                    ..Default::default()
                };
                Ok((input.to_vec(), Box::new(hit)))
            }
        }
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let hit = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        if let Some(reference) = hit.reference {
            reference.write_to(output)?;
            return Ok(());
        }
        if let Some(raw) = &hit.raw_data {
            output.write_all(raw)?;
            return Ok(());
        }
        output.write_all(&encode_model_hit(&hit, version)?)?;
        Ok(())
    }

    fn is_opaque(&self) -> bool {
        self.raw_data.is_some()
    }
}

fn decode_model_hit(input: &[u8], version: u16) -> Result<NuccChunkModelHit> {
    let mut reader = Reader::new(input);
    let object_count = reader.u32()? as usize;
    let triangle_count = reader.u32()? as usize;
    if object_count > input.len() / 8 {
        return Err(NuccError::InvalidChunk(
            "ModelHit object count exceeds payload".into(),
        ));
    }
    if triangle_count > input.len() / 36 + 1 {
        return Err(NuccError::InvalidChunk(
            "ModelHit triangle count exceeds payload".into(),
        ));
    }

    let wide = version > 0x79;
    let mut objects = Vec::with_capacity(object_count);
    let mut triangles = Vec::with_capacity(triangle_count);
    for _ in 0..object_count {
        let poly_count = reader.u32()?;
        let attribute_a = reader.u32()?;
        let (attribute_b, reserved) = if wide {
            (reader.u32()?, reader.u32()?)
        } else {
            (0, 0)
        };
        if triangles.len() + poly_count as usize > triangle_count {
            return Err(NuccError::InvalidChunk(
                "ModelHit object exceeds triangle count".into(),
            ));
        }
        objects.push(NuccModelHitObject {
            poly_count,
            attribute_a,
            attribute_b,
            reserved,
        });
        for _ in 0..poly_count {
            let mut v = [[0.0f32; 3]; 3];
            for point in &mut v {
                for coordinate in point {
                    *coordinate = reader.f32()?;
                }
            }
            triangles.push(NuccModelHitTriangle { v });
        }
    }
    if triangles.len() != triangle_count {
        return Err(NuccError::InvalidChunk(format!(
            "ModelHit decoded {} triangles, expected {triangle_count}",
            triangles.len()
        )));
    }

    Ok(NuccChunkModelHit {
        version,
        objects,
        triangles,
        trailing: reader.remaining().to_vec(),
        reference: None,
        raw_data: None,
    })
}

fn encode_model_hit(hit: &NuccChunkModelHit, version: u16) -> Result<Vec<u8>> {
    let wide = version > 0x79;
    let mut output = Vec::new();
    push_u32(&mut output, hit.objects.len() as u32);
    push_u32(&mut output, hit.triangles.len() as u32);
    let mut triangle_offset = 0usize;
    for object in &hit.objects {
        push_u32(&mut output, object.poly_count);
        push_u32(&mut output, object.attribute_a);
        if wide {
            push_u32(&mut output, object.attribute_b);
            push_u32(&mut output, object.reserved);
        }
        let end = triangle_offset + object.poly_count as usize;
        let triangles = hit
            .triangles
            .get(triangle_offset..end)
            .ok_or_else(|| NuccError::InvalidChunk("ModelHit triangle count is short".into()))?;
        for triangle in triangles {
            for point in &triangle.v {
                for coordinate in point {
                    output.extend_from_slice(&coordinate.to_be_bytes());
                }
            }
        }
        triangle_offset = end;
    }
    if triangle_offset != hit.triangles.len() {
        return Err(NuccError::InvalidChunk(
            "ModelHit object triangle counts do not match".into(),
        ));
    }
    output.extend_from_slice(&hit.trailing);
    Ok(output)
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
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

    fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn f32(&mut self) -> Result<f32> {
        let bytes = self.take(4)?;
        Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn remaining(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }
}
