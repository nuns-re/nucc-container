//! High-level XFBIN document built from raw container data.

use nucc_chunk::{NuccChunk, NuccChunkType, NuccChunkTypeExt, NuccChunkUnknown};
use nucc_core::{
    NuccChunkExternalIndex, NuccChunkIndex, NuccChunkInternalIndex, NuccError, Result,
};
use nucc_xfbin::{NuccXfbinChunkEntry, NuccXfbinChunkIndex, NuccXfbinData};
use std::str::FromStr;

#[derive(Debug)]
pub struct NuccChunkEntry {
    pub index: NuccChunkInternalIndex,
    pub chunk: Box<dyn NuccChunk>,
}

impl NuccChunkEntry {
    pub fn chunk_type(&self) -> NuccChunkType {
        self.index.chunk_type
    }

    pub fn chunk_name(&self) -> &str {
        &self.index.chunk_name
    }

    pub fn filepath(&self) -> &str {
        &self.index.filepath
    }

    pub fn is_typed(&self) -> bool {
        self.index.chunk_type != NuccChunkType::NuccChunkUnknown
            && self.chunk.chunk_type() == self.index.chunk_type
            && !self.chunk.is_opaque()
    }

    pub fn is_opaque(&self) -> bool {
        self.index.chunk_type == self.chunk.chunk_type() && self.chunk.is_opaque()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuccXfbinPage {
    pub chunk_index: usize,
    pub map_index_count: u32,
    pub reference_count: u32,
    pub chunks: Vec<usize>,
}

#[derive(Debug)]
pub struct NuccXfbinDocument {
    pub version: u32,
    pub index: NuccXfbinChunkIndex,
    pub chunks: Vec<NuccChunkEntry>,
    pub references: Vec<NuccChunkExternalIndex>,
    pub pages: Vec<NuccXfbinPage>,
}

impl NuccXfbinDocument {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let data = NuccXfbinData::parse(bytes)?;
        Self::from_data(data)
    }

    pub fn from_data(data: NuccXfbinData) -> Result<Self> {
        let entries = data.entries()?;
        let references = build_references(&data, &entries)?;
        let chunks = entries.into_iter().map(decode_entry).collect::<Vec<_>>();
        let pages = build_pages(&chunks);

        Ok(Self {
            version: data.header.version,
            index: data.index,
            chunks,
            references,
            pages,
        })
    }

    pub fn search_chunk(&self, chunk_type: NuccChunkType, name: &str) -> Option<&NuccChunkEntry> {
        self.chunks
            .iter()
            .find(|entry| entry.chunk_type() == chunk_type && entry.chunk_name() == name)
    }

    pub fn search_chunk_list(&self, chunk_type: NuccChunkType) -> Vec<&NuccChunkEntry> {
        self.chunks
            .iter()
            .filter(|entry| entry.chunk_type() == chunk_type)
            .collect()
    }

    pub fn is_owner(&self, index: &NuccChunkIndex) -> bool {
        self.chunks.iter().any(|entry| {
            entry.chunk_name() == index.chunk_name()
                && entry.chunk_type() == index.chunk_type()
                && entry.filepath() == index.filepath()
        })
    }
}

fn decode_entry(entry: NuccXfbinChunkEntry<'_>) -> NuccChunkEntry {
    let chunk_type = entry.index.chunk_type;
    let chunk = NuccChunkType::read_data(entry.data.to_vec(), chunk_type.as_str(), entry.version)
        .map(|(_, chunk)| chunk)
        .unwrap_or_else(|_| {
            Box::new(NuccChunkUnknown {
                version: entry.version,
                chunk_type: chunk_type.as_str().to_string(),
                data: entry.data.to_vec(),
            }) as Box<dyn NuccChunk>
        });

    NuccChunkEntry {
        index: entry.index,
        chunk,
    }
}

fn build_references(
    data: &NuccXfbinData,
    entries: &[NuccXfbinChunkEntry<'_>],
) -> Result<Vec<NuccChunkExternalIndex>> {
    let mut references = Vec::with_capacity(data.index.chunk_references.len());
    for reference in &data.index.chunk_references {
        let chunk_name = data
            .index
            .chunk_names
            .get(reference.chunk_name_index as usize)
            .ok_or_else(|| NuccError::InvalidIndex("reference name out of range".into()))?;
        let map = data
            .index
            .chunk_maps
            .get(reference.chunk_map_index as usize)
            .ok_or_else(|| NuccError::InvalidIndex("reference map out of range".into()))?;
        let chunk_type = data
            .index
            .chunk_types
            .get(map.chunk_type_index as usize)
            .map(|value| {
                NuccChunkType::from_str(&value.to_string())
                    .unwrap_or(NuccChunkType::NuccChunkUnknown)
            })
            .ok_or_else(|| NuccError::InvalidIndex("reference type out of range".into()))?;
        let filepath = data
            .index
            .filepaths
            .get(map.filepath_index as usize)
            .ok_or_else(|| NuccError::InvalidIndex("reference path out of range".into()))?;
        let link = entries
            .iter()
            .position(|entry| entry.index.chunk_name == chunk_name.to_string());

        references.push(NuccChunkExternalIndex {
            chunk_name: chunk_name.to_string(),
            chunk_type,
            filepath: filepath.to_string(),
            link,
            original_name: None,
            original_path: None,
        });
    }
    Ok(references)
}

fn build_pages(chunks: &[NuccChunkEntry]) -> Vec<NuccXfbinPage> {
    let mut pages = Vec::new();
    let mut current: Option<NuccXfbinPage> = None;

    for (index, entry) in chunks.iter().enumerate() {
        if entry.chunk_type() == NuccChunkType::NuccChunkPage {
            if let Some(page) = current.take() {
                pages.push(page);
            }
            let (map_index_count, reference_count) = entry
                .chunk
                .downcast_ref::<nucc_chunk::NuccChunkPage>()
                .map(|page| (page.map_index_count, page.reference_count))
                .unwrap_or((0, 0));
            current = Some(NuccXfbinPage {
                chunk_index: index,
                map_index_count,
                reference_count,
                chunks: vec![index],
            });
        } else if let Some(page) = current.as_mut() {
            page.chunks.push(index);
        }
    }

    if let Some(page) = current {
        pages.push(page);
    }
    pages
}
