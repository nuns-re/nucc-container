#![allow(unstable_name_collisions)]
#![allow(clippy::manual_is_multiple_of, clippy::collapsible_match)]

pub mod nucc_chunk;

pub use nucc_chunk::{
    NuccBillboardAnmData, NuccBillboardReference, NuccChunk, NuccChunkAmbient, NuccChunkAnm,
    NuccChunkAnmStrm, NuccChunkAnmStrmFrame, NuccChunkBillboard, NuccChunkBinary, NuccChunkCamera,
    NuccChunkClump, NuccChunkCoord, NuccChunkDynamics, NuccChunkLayerSet, NuccChunkLightDirc,
    NuccChunkLightPoint, NuccChunkMaterial, NuccChunkModel, NuccChunkMorphModel, NuccChunkNub,
    NuccChunkNull, NuccChunkPage, NuccChunkParticle, NuccChunkTexture, NuccChunkTrail,
    NuccChunkType, NuccChunkTypeExt, NuccChunkUnknown,
};

#[cfg(feature = "nus3")]
pub use nucc_nus3::{Nus3Bank, Nus3Chunk, Nus3Codec, Nus3Tone};

#[cfg(feature = "acb")]
pub use nucc_acb::{
    Acb, AcbBlock, AcbBlockSequence, AcbCue, AcbCueName, AcbHeader, AcbReference, AcbSequence,
    AcbSynth, AcbTrack, AcbTrackEvent, AcbWaveform, AcbWaveformExtension, UtfTable, UtfValue,
};

#[cfg(feature = "awb")]
pub use nucc_awb::{Awb, AwbCodec, AwbEntry, AwbHeader};
