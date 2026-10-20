#![allow(unstable_name_collisions)]
#![allow(clippy::manual_is_multiple_of, clippy::collapsible_match)]

pub mod nucc_chunk;

pub use nucc_chunk::{
    NuccChunk, NuccChunkAmbient, NuccChunkAnm, NuccChunkAnmStrm, NuccChunkAnmStrmFrame,
    NuccChunkBinary, NuccChunkCamera, NuccChunkLayerSet, NuccChunkLightDirc, NuccChunkLightPoint,
    NuccChunkMorphModel, NuccChunkNull, NuccChunkPage, NuccChunkParticle, NuccChunkType,
    NuccChunkTypeExt, NuccChunkUnknown,
};
