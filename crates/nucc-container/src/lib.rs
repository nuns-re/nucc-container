//! High-level document layer for NUCC XFBIN files.

mod document;

pub use document::{NuccChunkEntry, NuccXfbinDocument, NuccXfbinPage};
pub use nucc_chunk::{NuccChunkModel, NuccChunkNub, NuccChunkTexture, NuccChunkTrail};

#[cfg(feature = "nus3")]
pub use nucc_chunk::{Nus3Bank, Nus3Chunk, Nus3Codec, Nus3Tone};

#[cfg(feature = "acb")]
pub use nucc_chunk::{
    Acb, AcbBlock, AcbBlockSequence, AcbCue, AcbCueName, AcbHeader, AcbReference, AcbSequence,
    AcbSynth, AcbTrack, AcbTrackEvent, AcbWaveform, AcbWaveformExtension, UtfTable, UtfValue,
};

#[cfg(feature = "awb")]
pub use nucc_chunk::{Awb, AwbCodec, AwbEntry, AwbHeader};
