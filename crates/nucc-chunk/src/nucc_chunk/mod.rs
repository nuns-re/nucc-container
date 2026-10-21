pub mod nucc_chunk_ambient;
pub mod nucc_chunk_anm;
pub mod nucc_chunk_anmstrm;
pub mod nucc_chunk_anmstrmframe;
pub mod nucc_chunk_billboard;
pub mod nucc_chunk_binary;
pub mod nucc_chunk_camera;
pub mod nucc_chunk_clump;
pub mod nucc_chunk_coord;
pub mod nucc_chunk_dynamics;
pub mod nucc_chunk_layerset;
pub mod nucc_chunk_lightdirc;
pub mod nucc_chunk_lightpoint;
pub mod nucc_chunk_material;
pub mod nucc_chunk_model;
pub mod nucc_chunk_morphmodel;
pub mod nucc_chunk_nub;
pub mod nucc_chunk_null;
pub mod nucc_chunk_page;
pub mod nucc_chunk_particle;
pub mod nucc_chunk_texture;
pub mod nucc_chunk_trail;
pub mod nucc_chunk_unknown;

use binrw::{io::Cursor, BinRead, BinWrite};
use downcast_rs::{impl_downcast, Downcast};
use std::error::Error;
use std::fmt;
use std::io::Write;
use std::str::FromStr;

pub use nucc_chunk_ambient::NuccChunkAmbient;
pub use nucc_chunk_anm::NuccChunkAnm;
pub use nucc_chunk_anmstrm::NuccChunkAnmStrm;
pub use nucc_chunk_anmstrmframe::NuccChunkAnmStrmFrame;
pub use nucc_chunk_billboard::{NuccBillboardAnmData, NuccChunkBillboard};
pub use nucc_chunk_binary::NuccChunkBinary;
pub use nucc_chunk_camera::NuccChunkCamera;
pub use nucc_chunk_clump::NuccChunkClump;
pub use nucc_chunk_coord::NuccChunkCoord;
pub use nucc_chunk_dynamics::{DynamicsCollisionSphere, DynamicsSpringGroup, NuccChunkDynamics};
pub use nucc_chunk_layerset::NuccChunkLayerSet;
pub use nucc_chunk_lightdirc::NuccChunkLightDirc;
pub use nucc_chunk_lightpoint::NuccChunkLightPoint;
pub use nucc_chunk_material::{MaterialTextureGroup, NuccChunkMaterial};
pub use nucc_chunk_model::NuccChunkModel;
pub use nucc_chunk_morphmodel::NuccChunkMorphModel;
pub use nucc_chunk_nub::NuccChunkNub;
pub use nucc_chunk_null::NuccChunkNull;
pub use nucc_chunk_page::NuccChunkPage;
pub use nucc_chunk_particle::NuccChunkParticle;
pub use nucc_chunk_texture::NuccChunkTexture;
pub use nucc_chunk_trail::NuccChunkTrail;
pub use nucc_chunk_unknown::NuccChunkUnknown;
pub use nucc_core::NuccChunkType;

pub type NuccChunkReadResult = std::result::Result<(Vec<u8>, Box<dyn NuccChunk>), Box<dyn Error>>;
pub type NuccChunkWriteResult = std::result::Result<Vec<u8>, Box<dyn Error>>;

pub trait NuccChunk: Downcast + fmt::Debug {
    fn chunk_type(&self) -> NuccChunkType;
    fn version(&self) -> u16;
    fn extension(&self) -> String {
        String::new()
    }

    fn is_opaque(&self) -> bool {
        false
    }

    fn read_boxed(input: &[u8], version: u16) -> NuccChunkReadResult
    where
        Self: Sized + BinRead<Args = (u16,)>,
    {
        let mut cursor = Cursor::new(input);
        let result = Self::read_be_args(&mut cursor, (version,))?;
        Ok((input.to_vec(), Box::new(result) as Box<dyn NuccChunk>))
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut Cursor<Vec<u8>>,
        version: u16,
    ) -> std::result::Result<(), Box<dyn Error>>
    where
        Self: Sized + BinWrite<Args = (u16,)>,
    {
        let chunk = boxed
            .downcast::<Self>()
            .map(|chunk| *chunk)
            .map_err(|_| "chunk type mismatch")?;
        Self::write_be_args(&chunk, output, (version,))?;
        Ok(())
    }
}

impl_downcast!(NuccChunk);

pub trait NuccChunkTypeExt {
    fn read_data(data: Vec<u8>, chunk_type: &str, version: u16) -> NuccChunkReadResult;

    fn write_data(boxed: Box<dyn NuccChunk>, version: u16) -> NuccChunkWriteResult;
}

impl NuccChunkTypeExt for NuccChunkType {
    fn read_data(data: Vec<u8>, chunk_type: &str, version: u16) -> NuccChunkReadResult {
        match NuccChunkType::from_str(chunk_type).unwrap_or(NuccChunkType::NuccChunkUnknown) {
            NuccChunkType::NuccChunkNull => Ok((data, Box::new(NuccChunkNull(version)))),
            NuccChunkType::NuccChunkPage => NuccChunkPage::read_boxed(&data, version),
            NuccChunkType::NuccChunkBillboard => NuccChunkBillboard::read_boxed(&data, version),
            NuccChunkType::NuccChunkBinary => NuccChunkBinary::read_boxed(&data, version),
            NuccChunkType::NuccChunkAnm => NuccChunkAnm::read_boxed(&data, version),
            NuccChunkType::NuccChunkParticle => NuccChunkParticle::read_boxed(&data, version),
            NuccChunkType::NuccChunkAnmStrm => NuccChunkAnmStrm::read_boxed(&data, version),
            NuccChunkType::NuccChunkAnmStrmFrame => {
                NuccChunkAnmStrmFrame::read_boxed(&data, version)
            }
            NuccChunkType::NuccChunkCamera => NuccChunkCamera::read_boxed(&data, version),
            NuccChunkType::NuccChunkClump => NuccChunkClump::read_boxed(&data, version),
            NuccChunkType::NuccChunkCoord => NuccChunkCoord::read_boxed(&data, version),
            NuccChunkType::NuccChunkDynamics => NuccChunkDynamics::read_boxed(&data, version),
            NuccChunkType::NuccChunkMaterial => NuccChunkMaterial::read_boxed(&data, version),
            NuccChunkType::NuccChunkModel => NuccChunkModel::read_boxed(&data, version),
            NuccChunkType::NuccChunkTexture => NuccChunkTexture::read_boxed(&data, version),
            NuccChunkType::NuccChunkTrail => NuccChunkTrail::read_boxed(&data, version),
            NuccChunkType::NuccChunkLightDirc => NuccChunkLightDirc::read_boxed(&data, version),
            NuccChunkType::NuccChunkLightPoint => NuccChunkLightPoint::read_boxed(&data, version),
            NuccChunkType::NuccChunkLayerSet => NuccChunkLayerSet::read_boxed(&data, version),
            NuccChunkType::NuccChunkAmbient => NuccChunkAmbient::read_boxed(&data, version),
            NuccChunkType::NuccChunkMorphModel => NuccChunkMorphModel::read_boxed(&data, version),
            NuccChunkType::NuccChunkNub => NuccChunkNub::read_boxed(&data, version),
            _ => Ok((
                data.clone(),
                Box::new(NuccChunkUnknown {
                    version,
                    chunk_type: chunk_type.to_string(),
                    data,
                }),
            )),
        }
    }

    fn write_data(boxed: Box<dyn NuccChunk>, version: u16) -> NuccChunkWriteResult {
        let mut output = Cursor::new(Vec::new());
        match boxed.chunk_type() {
            NuccChunkType::NuccChunkNull | NuccChunkType::NuccChunkIndex => {
                return Ok(output.into_inner());
            }
            NuccChunkType::NuccChunkPage => {
                NuccChunkPage::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkBillboard => {
                NuccChunkBillboard::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkBinary => {
                NuccChunkBinary::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkAnm => NuccChunkAnm::write_boxed(boxed, &mut output, version)?,
            NuccChunkType::NuccChunkParticle => {
                NuccChunkParticle::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkAnmStrm => {
                NuccChunkAnmStrm::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkAnmStrmFrame => {
                NuccChunkAnmStrmFrame::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkCamera => {
                NuccChunkCamera::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkClump => {
                NuccChunkClump::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkCoord => {
                NuccChunkCoord::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkDynamics => {
                NuccChunkDynamics::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkMaterial => {
                NuccChunkMaterial::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkModel => {
                NuccChunkModel::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkTexture => {
                NuccChunkTexture::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkTrail => {
                NuccChunkTrail::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkLightDirc => {
                NuccChunkLightDirc::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkLightPoint => {
                NuccChunkLightPoint::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkLayerSet => {
                NuccChunkLayerSet::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkAmbient => {
                NuccChunkAmbient::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkMorphModel => {
                NuccChunkMorphModel::write_boxed(boxed, &mut output, version)?
            }
            NuccChunkType::NuccChunkNub => NuccChunkNub::write_boxed(boxed, &mut output, version)?,
            NuccChunkType::NuccChunkUnknown => {
                let unknown = boxed
                    .downcast::<NuccChunkUnknown>()
                    .map(|chunk| chunk.data)
                    .map_err(|_| "chunk type mismatch")?;
                output.write_all(&unknown)?;
            }
            _ => return Err("chunk type has no writer".into()),
        }
        Ok(output.into_inner())
    }
}
