use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkNub {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    pub size: u32,
    #[br(count = size)]
    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkNub {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkNub
    }

    fn version(&self) -> u16 {
        self.version
    }
}

#[cfg(feature = "nu")]
impl NuccChunkNub {
    pub fn bank(&self) -> nucc_core::Result<nu::audio::Nus2Bank> {
        nu::audio::Nus2Bank::parse(&self.data)
            .map_err(|error| nucc_core::NuccError::InvalidChunk(format!("NUB: {error}")))
    }
}

#[cfg(feature = "nus3")]
impl NuccChunkNub {
    pub fn nus3bank(&self) -> nucc_core::Result<nucc_nus3::Nus3Bank> {
        nucc_nus3::Nus3Bank::parse(&self.data)
    }
}

impl NuccChunkNub {
    pub fn payload(&self) -> &[u8] {
        &self.data
    }
}
