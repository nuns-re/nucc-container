use binrw::NullString;
use nucc_xfbin::{
    NuccXfbinChunk, NuccXfbinChunkIndex, NuccXfbinChunkMap, NuccXfbinData, NuccXfbinHeader,
};

fn sample() -> NuccXfbinData {
    NuccXfbinData {
        header: NuccXfbinHeader {
            version: 121,
            encrypted: 0,
        },
        index: NuccXfbinChunkIndex {
            min_page_size: 0,
            version: 1,
            chunk_types: vec![NullString::from("nuccChunkBinary")],
            filepaths: vec![NullString::from("data/test.bin")],
            chunk_names: vec![NullString::from("test")],
            chunk_maps: vec![NuccXfbinChunkMap {
                chunk_type_index: 0,
                filepath_index: 0,
                chunk_name_index: 0,
            }],
            chunk_references: Vec::new(),
            chunk_map_indices: vec![0],
        },
        chunks: vec![NuccXfbinChunk {
            chunk_map_index: 0,
            version: 1,
            data: vec![1, 2, 3, 4],
        }],
    }
}

#[test]
fn parses_and_lists_chunks() {
    let bytes = sample().to_bytes().unwrap();
    let parsed = NuccXfbinData::parse(&bytes).unwrap();
    assert_eq!(parsed.header.version, 121);
    assert_eq!(parsed.chunks.len(), 1);

    let entries = parsed.entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].index.chunk_name, "test");
    assert_eq!(entries[0].index.filepath, "data/test.bin");
    assert_eq!(entries[0].data, &[1, 2, 3, 4]);
}

#[test]
fn resolves_page_local_chunk_maps() {
    let mut page_data = 2u32.to_be_bytes().to_vec();
    page_data.extend_from_slice(&0u32.to_be_bytes());
    let data = NuccXfbinData {
        header: NuccXfbinHeader {
            version: 121,
            encrypted: 0,
        },
        index: NuccXfbinChunkIndex {
            min_page_size: 0,
            version: 1,
            chunk_types: vec![
                NullString::from("nuccChunkBinary"),
                NullString::from("nuccChunkPage"),
            ],
            filepaths: vec![NullString::from("data/test.bin")],
            chunk_names: vec![NullString::from("data"), NullString::from("Page0")],
            chunk_maps: vec![
                NuccXfbinChunkMap {
                    chunk_type_index: 0,
                    filepath_index: 0,
                    chunk_name_index: 0,
                },
                NuccXfbinChunkMap {
                    chunk_type_index: 1,
                    filepath_index: 0,
                    chunk_name_index: 1,
                },
            ],
            chunk_references: Vec::new(),
            chunk_map_indices: vec![0, 1],
        },
        chunks: vec![
            NuccXfbinChunk {
                chunk_map_index: 0,
                version: 121,
                data: vec![9, 8, 7, 6],
            },
            NuccXfbinChunk {
                chunk_map_index: 1,
                version: 121,
                data: page_data,
            },
        ],
    };

    let bytes = data.to_bytes().unwrap();
    let parsed = NuccXfbinData::parse(&bytes).unwrap();
    let entries = parsed.entries().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].index.chunk_name, "data");
    assert_eq!(entries[0].data, &[9, 8, 7, 6]);
    assert_eq!(entries[1].index.chunk_type.as_str(), "nuccChunkPage");
}
