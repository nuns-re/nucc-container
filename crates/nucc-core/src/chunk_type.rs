//! NUCC chunk type identifiers.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NuccChunkType {
    NuccChunkNull,
    NuccChunkIndex,
    NuccChunkPage,
    NuccChunkBinary,
    NuccChunkAnm,
    NuccChunkAnmStrm,
    NuccChunkAnmStrmFrame,
    NuccChunkParticle,
    NuccChunkCamera,
    NuccChunkAmbient,
    NuccChunkBillboard,
    NuccChunkClump,
    NuccChunkCoord,
    NuccChunkDynamics,
    NuccChunkFont,
    NuccChunkLayerSet,
    NuccChunkLightDirc,
    NuccChunkLightPoint,
    NuccChunkLightSet,
    NuccChunkMaterial,
    NuccChunkModel,
    NuccChunkModelHit,
    NuccChunkModelVertex,
    NuccChunkModelPrimitiveBatch,
    NuccChunkMorphModel,
    NuccChunkMorphPrimitive,
    NuccChunkNub,
    NuccChunkPostFilter,
    NuccChunkPrimitiveVertex,
    NuccChunkSprite,
    NuccChunkSprite2,
    NuccChunkSpriteAnm,
    NuccChunkSprite2Anm,
    NuccChunkTexture,
    NuccChunkTrail,
    NuccChunkUnknown,
}

impl NuccChunkType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NuccChunkNull => "nuccChunkNull",
            Self::NuccChunkIndex => "nuccChunkIndex",
            Self::NuccChunkPage => "nuccChunkPage",
            Self::NuccChunkBinary => "nuccChunkBinary",
            Self::NuccChunkAnm => "nuccChunkAnm",
            Self::NuccChunkAnmStrm => "nuccChunkAnmStrm",
            Self::NuccChunkAnmStrmFrame => "nuccChunkAnmStrmFrame",
            Self::NuccChunkParticle => "nuccChunkParticle",
            Self::NuccChunkCamera => "nuccChunkCamera",
            Self::NuccChunkAmbient => "nuccChunkAmbient",
            Self::NuccChunkBillboard => "nuccChunkBillboard",
            Self::NuccChunkClump => "nuccChunkClump",
            Self::NuccChunkCoord => "nuccChunkCoord",
            Self::NuccChunkDynamics => "nuccChunkDynamics",
            Self::NuccChunkFont => "nuccChunkFont",
            Self::NuccChunkLayerSet => "nuccChunkLayerSet",
            Self::NuccChunkLightDirc => "nuccChunkLightDirc",
            Self::NuccChunkLightPoint => "nuccChunkLightPoint",
            Self::NuccChunkLightSet => "nuccChunkLightSet",
            Self::NuccChunkMaterial => "nuccChunkMaterial",
            Self::NuccChunkModel => "nuccChunkModel",
            Self::NuccChunkModelHit => "nuccChunkModelHit",
            Self::NuccChunkModelVertex => "nuccChunkModelVertex",
            Self::NuccChunkModelPrimitiveBatch => "nuccChunkModelPrimitiveBatch",
            Self::NuccChunkMorphModel => "nuccChunkMorphModel",
            Self::NuccChunkMorphPrimitive => "nuccChunkMorphPrimitive",
            Self::NuccChunkNub => "nuccChunkNub",
            Self::NuccChunkPostFilter => "nuccChunkPostFilter",
            Self::NuccChunkPrimitiveVertex => "nuccChunkPrimitiveVertex",
            Self::NuccChunkSprite => "nuccChunkSprite",
            Self::NuccChunkSprite2 => "nuccChunkSprite2",
            Self::NuccChunkSpriteAnm => "nuccChunkSpriteAnm",
            Self::NuccChunkSprite2Anm => "nuccChunkSprite2Anm",
            Self::NuccChunkTexture => "nuccChunkTexture",
            Self::NuccChunkTrail => "nuccChunkTrail",
            Self::NuccChunkUnknown => "nuccChunkUnknown",
        }
    }
}

impl fmt::Display for NuccChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NuccChunkType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim_start_matches("nuccChunk");
        Ok(match value.to_ascii_lowercase().as_str() {
            "null" => Self::NuccChunkNull,
            "index" => Self::NuccChunkIndex,
            "page" => Self::NuccChunkPage,
            "binary" => Self::NuccChunkBinary,
            "anm" => Self::NuccChunkAnm,
            "anmstrm" => Self::NuccChunkAnmStrm,
            "anmstrmframe" => Self::NuccChunkAnmStrmFrame,
            "particle" => Self::NuccChunkParticle,
            "camera" => Self::NuccChunkCamera,
            "ambient" => Self::NuccChunkAmbient,
            "billboard" => Self::NuccChunkBillboard,
            "clump" => Self::NuccChunkClump,
            "coord" => Self::NuccChunkCoord,
            "dynamics" => Self::NuccChunkDynamics,
            "font" => Self::NuccChunkFont,
            "layerset" => Self::NuccChunkLayerSet,
            "lightdirc" => Self::NuccChunkLightDirc,
            "lightpoint" => Self::NuccChunkLightPoint,
            "lightset" => Self::NuccChunkLightSet,
            "material" => Self::NuccChunkMaterial,
            "model" => Self::NuccChunkModel,
            "modelhit" => Self::NuccChunkModelHit,
            "modelvertex" => Self::NuccChunkModelVertex,
            "modelprimitivebatch" => Self::NuccChunkModelPrimitiveBatch,
            "morphmodel" => Self::NuccChunkMorphModel,
            "morphprimitive" => Self::NuccChunkMorphPrimitive,
            "nub" => Self::NuccChunkNub,
            "postfilter" => Self::NuccChunkPostFilter,
            "primitivevertex" => Self::NuccChunkPrimitiveVertex,
            "sprite" => Self::NuccChunkSprite,
            "sprite2" => Self::NuccChunkSprite2,
            "spriteanm" => Self::NuccChunkSpriteAnm,
            "sprite2anm" => Self::NuccChunkSprite2Anm,
            "texture" => Self::NuccChunkTexture,
            "trail" => Self::NuccChunkTrail,
            "unknown" => Self::NuccChunkUnknown,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::NuccChunkType;
    use std::str::FromStr;

    #[test]
    fn names_round_trip() {
        for chunk_type in [
            NuccChunkType::NuccChunkAnm,
            NuccChunkType::NuccChunkBinary,
            NuccChunkType::NuccChunkIndex,
            NuccChunkType::NuccChunkModel,
            NuccChunkType::NuccChunkTexture,
            NuccChunkType::NuccChunkUnknown,
        ] {
            assert_eq!(NuccChunkType::from_str(chunk_type.as_str()), Ok(chunk_type));
        }
    }
}
