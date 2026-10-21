//! Checked XFBIN parsing.

use binrw::{BinReaderExt, BinWrite};
use nucc_core::{NuccChunkInternalIndex, NuccChunkType, NuccError, Result};
use std::io::Cursor;
use std::str::FromStr;

use crate::data::{NuccXfbinChunk, NuccXfbinChunkIndex, NuccXfbinData, NuccXfbinHeader};

#[derive(Debug, Clone)]
pub struct NuccXfbinChunkEntry<'a> {
    pub index: NuccChunkInternalIndex,
    pub version: u16,
    pub data: &'a [u8],
}

impl NuccXfbinData {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        NuccXfbinParser::parse(bytes)
    }

    pub fn entries(&self) -> Result<Vec<NuccXfbinChunkEntry<'_>>> {
        let index = &self.index;
        let mut entries = Vec::with_capacity(self.chunks.len());
        let mut base = 0usize;

        for chunk in &self.chunks {
            let local = chunk.chunk_map_index as usize;
            let global = base.checked_add(local).ok_or(NuccError::Overflow)?;
            let map_index = *index.chunk_map_indices.get(global).ok_or_else(|| {
                NuccError::InvalidIndex(format!("page map index {global} is out of range"))
            })? as usize;
            let map = index.chunk_maps.get(map_index).ok_or_else(|| {
                NuccError::InvalidIndex(format!("chunk map {map_index} is out of range"))
            })?;

            let chunk_type_name = index
                .chunk_types
                .get(map.chunk_type_index as usize)
                .ok_or_else(|| NuccError::InvalidIndex("chunk type is out of range".into()))?;
            let filepath = index
                .filepaths
                .get(map.filepath_index as usize)
                .ok_or_else(|| NuccError::InvalidIndex("filepath is out of range".into()))?;
            let chunk_name = index
                .chunk_names
                .get(map.chunk_name_index as usize)
                .ok_or_else(|| NuccError::InvalidIndex("chunk name is out of range".into()))?;

            let type_name = chunk_type_name.to_string();
            let chunk_type =
                NuccChunkType::from_str(&type_name).unwrap_or(NuccChunkType::NuccChunkUnknown);
            let is_page = type_name == NuccChunkType::NuccChunkPage.as_str();
            entries.push(NuccXfbinChunkEntry {
                index: NuccChunkInternalIndex {
                    chunk_name: chunk_name.to_string(),
                    chunk_type,
                    filepath: filepath.to_string(),
                    link: None,
                    original_name: None,
                    original_path: None,
                },
                version: chunk.version,
                data: &chunk.data,
            });

            if is_page {
                let page_count = chunk
                    .data
                    .get(..4)
                    .and_then(|bytes| bytes.try_into().ok())
                    .map(u32::from_be_bytes)
                    .ok_or_else(|| NuccError::InvalidChunk("truncated page header".into()))?;
                if page_count == 0 || local >= page_count as usize {
                    return Err(NuccError::InvalidIndex(
                        "page chunk table exceeds page length".into(),
                    ));
                }
                base = base
                    .checked_add(page_count as usize)
                    .ok_or(NuccError::Overflow)?;
            }
        }

        Ok(entries)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::new());
        BinWrite::write_be(self, &mut cursor)
            .map_err(|error| NuccError::InvalidChunk(error.to_string()))?;
        Ok(cursor.into_inner())
    }
}

pub struct NuccXfbinParser;

impl NuccXfbinParser {
    pub fn parse(bytes: &[u8]) -> Result<NuccXfbinData> {
        if bytes.len() < 16 || &bytes[..4] != b"NUCC" {
            return Err(NuccError::InvalidHeader);
        }
        if bytes.len() > 512 * 1024 * 1024 {
            return Err(NuccError::Unsupported("XFBIN larger than 512 MiB".into()));
        }

        let mut cursor = Cursor::new(bytes);
        let header: NuccXfbinHeader = cursor.read_be().map_err(|_| NuccError::InvalidHeader)?;
        if header.encrypted != 0 {
            return Err(NuccError::Unsupported("encrypted XFBIN".into()));
        }

        let index: NuccXfbinChunkIndex = cursor
            .read_be()
            .map_err(|error| NuccError::InvalidIndex(error.to_string()))?;

        let mut budget = 0usize;
        for (count, stride) in [
            (index.chunk_types.len(), 1usize),
            (index.filepaths.len(), 1),
            (index.chunk_names.len(), 1),
            (index.chunk_maps.len(), 12),
            (index.chunk_map_indices.len(), 4),
            (index.chunk_references.len(), 8),
        ] {
            budget = budget
                .checked_add(count.checked_mul(stride).ok_or(NuccError::Overflow)?)
                .ok_or(NuccError::Overflow)?;
        }
        if budget > bytes.len() {
            return Err(NuccError::InvalidIndex(
                "index counts exceed file size".into(),
            ));
        }

        for map in &index.chunk_maps {
            if map.chunk_type_index as usize >= index.chunk_types.len()
                || map.filepath_index as usize >= index.filepaths.len()
                || map.chunk_name_index as usize >= index.chunk_names.len()
            {
                return Err(NuccError::InvalidIndex(
                    "chunk map index out of range".into(),
                ));
            }
        }

        let mut chunks = Vec::new();
        while (cursor.position() as usize) < bytes.len() {
            let chunk: NuccXfbinChunk = cursor
                .read_be()
                .map_err(|error| NuccError::InvalidChunk(error.to_string()))?;
            if chunk.chunk_map_index as usize >= index.chunk_maps.len() {
                return Err(NuccError::InvalidChunk(format!(
                    "chunk map index {} out of range",
                    chunk.chunk_map_index
                )));
            }
            chunks.push(chunk);
        }

        Ok(NuccXfbinData {
            header,
            index,
            chunks,
        })
    }
}
