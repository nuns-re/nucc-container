//! NUT integration for `nuccChunkTexture`.

use nucc_chunk::NuccChunkTexture;
use nucc_core::{NuccError, Result};

pub trait NuccChunkTextureExt {
    fn nut(&self) -> Result<nu::texture::Nut>;
}

impl NuccChunkTextureExt for NuccChunkTexture {
    fn nut(&self) -> Result<nu::texture::Nut> {
        nu::texture::Nut::parse(&self.nut_data)
            .map_err(|error| NuccError::InvalidChunk(format!("NUT: {error}")))
    }
}
