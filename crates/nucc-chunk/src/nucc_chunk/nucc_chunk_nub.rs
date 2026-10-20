use binrw::binrw;

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big, import(version: u16))]
#[derive(Debug, Clone, PartialEq)]
pub struct NuccChunkNub {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,
    #[br(parse_with = binrw::until_eof)]
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
