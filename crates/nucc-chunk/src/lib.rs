#![allow(unstable_name_collisions)]
#![allow(clippy::manual_is_multiple_of, clippy::collapsible_match)]

pub mod nucc_chunk;

pub use nucc_chunk::{
    NuccChunk, NuccChunkAmbient, NuccChunkAnm, NuccChunkAnmStrm, NuccChunkAnmStrmFrame,
    NuccChunkBinary, NuccChunkCamera, NuccChunkClump, NuccChunkCoord, NuccChunkDynamics,
    NuccChunkLayerSet, NuccChunkLightDirc, NuccChunkLightPoint, NuccChunkMaterial, NuccChunkModel,
    NuccChunkMorphModel, NuccChunkNub, NuccChunkNull, NuccChunkPage, NuccChunkParticle,
    NuccChunkTexture, NuccChunkType, NuccChunkTypeExt, NuccChunkUnknown,
};
