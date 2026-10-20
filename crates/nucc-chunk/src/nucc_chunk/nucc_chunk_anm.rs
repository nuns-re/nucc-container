use binrw::{binrw, BinRead, BinResult, ReadOptions};
use std::{
    default,
    io::{Read, Seek},
};

use super::{NuccChunk, NuccChunkType};

#[binrw]
#[brw(big)]
#[brw(import(version: u16))]
#[derive(Debug, Clone, Default)]
pub struct NuccChunkAnm {
    #[br(calc = version)]
    #[bw(ignore)]
    pub version: u16,

    pub frame_count: u32,

    #[bw(calc = 100)]
    pub frame_size: u32,

    #[bw(calc = entries.len() as u16)]
    pub entry_count: u16,

    pub is_looped: u16,

    #[bw(calc = clumps.len() as u16)]
    pub clump_count: u16,

    #[bw(calc = other_entries_indices.len() as u16)]
    pub other_entry_count: u16,

    #[bw(calc = unk_entry_indices.len() as u16)]
    pub unk_entry_count: u16,

    #[bw(calc = coord_parents.len() as u16)]
    pub coord_count: u16,

    #[br(count = clump_count)]
    pub clumps: Vec<AnmClump>,

    #[br(count = other_entry_count)]
    pub other_entries_indices: Vec<u32>,

    #[br(count = unk_entry_count)]
    pub unk_entry_indices: Vec<u32>,

    #[br(count = coord_count)]
    pub coord_parents: Vec<CoordParent>,

    #[br(count = entry_count)]
    pub entries: Vec<AnmEntry>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmClump {
    pub clump_index: u32,

    #[bw(calc = bone_material_indices.len() as u16)]
    pub bone_material_count: u16,

    #[bw(calc = model_indices.len() as u16)]
    pub model_count: u16,

    #[br(count = bone_material_count)]
    pub bone_material_indices: Vec<u32>,

    #[br(count = model_count)]
    pub model_indices: Vec<u32>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct CoordParent {
    pub parent: AnmCoord,
    pub child: AnmCoord,
}

#[binrw]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnmCoord {
    pub clump_index: i16,
    pub coord_index: u16,
}

#[binrw]
#[brw(repr(u16))]
#[derive(Debug, Clone, Default)]
pub enum EntryFormat {
    Coord = 1,
    Camera = 2,
    Material = 4,
    LightDirc = 5,
    LightPoint = 6,
    Ambient = 8,
    MorphModel = 9,

    #[default]
    Unknown,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct AnmEntry {
    pub coord: AnmCoord,

    pub entry_format: EntryFormat,

    #[bw(calc = track_headers.len() as u16)]
    pub track_count: u16,

    #[br(count = track_count)]
    pub track_headers: Vec<TrackHeader>,

    #[br(parse_with = read_tracks(track_headers.iter()))]
    pub tracks: Vec<AnmTrack>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackHeader {
    pub track_index: u16,
    pub key_format: NuccAnmKeyFormat,
    pub frame_count: u16,

    #[bw(calc = key_format.size_per_frame() as u16 * frame_count)]
    pub track_size: u16,
}

#[binrw]
#[brw(repr(u16))]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NuccAnmKeyFormat {
    Vector3Fixed = 0x5,
    Vector3Linear = 0x6,
    Vector3Bezier = 0x7,
    EulerXYZFixed = 0x8,
    EulerInterpolated = 0x9,
    QuaternionLinear = 0xA,
    FloatFixed = 0xB,
    FloatLinear = 0xC,
    Vector2Fixed = 0xD,
    Vector2Linear = 0xE,
    OpacityShortTable = 0xF,
    ScaleShortTable = 0x10,
    QuaternionShortTable = 0x11,
    ColorRGBTable = 0x14,
    Vector3Table = 0x15,
    FloatTable = 0x16,
    QuaternionTable = 0x17,
    FloatTableNoInterp = 0x18,
    Vector3ShortLinear = 0x19,
    Vector3TableNoInterp = 0x1A,
    QuaternionShortTableNoInterp = 0x1B,
    OpacityShortTableNoInterp = 0x1D,

    #[default]
    Unknown,
}

impl NuccAnmKeyFormat {
    pub fn size_per_frame(&self) -> usize {
        match self {
            NuccAnmKeyFormat::OpacityShortTable | NuccAnmKeyFormat::OpacityShortTableNoInterp => {
                0x02
            }
            NuccAnmKeyFormat::ColorRGBTable => 0x03,
            NuccAnmKeyFormat::FloatFixed
            | NuccAnmKeyFormat::FloatTable
            | NuccAnmKeyFormat::FloatTableNoInterp => 0x04,

            NuccAnmKeyFormat::ScaleShortTable => 0x06,
            NuccAnmKeyFormat::FloatLinear
            | NuccAnmKeyFormat::Vector2Fixed
            | NuccAnmKeyFormat::QuaternionShortTable
            | NuccAnmKeyFormat::QuaternionShortTableNoInterp => 0x08,
            NuccAnmKeyFormat::Vector3Fixed
            | NuccAnmKeyFormat::EulerXYZFixed
            | NuccAnmKeyFormat::Vector2Linear
            | NuccAnmKeyFormat::Vector3Table
            | NuccAnmKeyFormat::Vector3TableNoInterp => 0x0C,
            NuccAnmKeyFormat::Vector3Linear | NuccAnmKeyFormat::QuaternionTable => 0x10,
            NuccAnmKeyFormat::QuaternionLinear => 0x14,
            NuccAnmKeyFormat::Vector3Bezier => 0x10,
            NuccAnmKeyFormat::EulerInterpolated => 0x0C,
            NuccAnmKeyFormat::Vector3ShortLinear => 0x0C,
            NuccAnmKeyFormat::Unknown => 0x0C,
        }
    }
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub enum NuccAnmKey {
    Vec3 {
        values: (f32, f32, f32),
    },
    Vec3Linear {
        frame: i32,
        values: (f32, f32, f32),
    },
    Vec4Linear {
        frame: i32,
        values: (f32, f32, f32, f32),
    },
    Float {
        values: f32,
    },
    FloatLinear {
        frame: i32,
        values: f32,
    },
    I16Vec {
        values: i16,
    },
    I16Vec3 {
        values: (i16, i16, i16),
    },
    ShortVec4 {
        values: (i16, i16, i16, i16),
    }, // QuaternionShortTable
    Color {
        values: (u8, u8, u8),
    }, // ColorRGBTable

    Unknown {},
}

impl default::Default for NuccAnmKey {
    fn default() -> Self {
        NuccAnmKey::Unknown {}
    }
}

#[binrw]
#[br(import_raw(header: TrackHeader))]
#[derive(Debug, Clone, PartialEq)]
pub struct AnmTrack {
    #[br(parse_with = |r, o, _h: TrackHeader| read_track_data(r, o, header))]
    pub keys: Vec<NuccAnmKey>,
}

impl AnmTrack {
    pub fn pad_values(&mut self, key_format: &NuccAnmKeyFormat) {
        match key_format {
            NuccAnmKeyFormat::OpacityShortTable => {
                if self.keys.len() % 2 != 0 {
                    let last_value = self.keys.last().unwrap().clone();

                    for _ in self.keys.len() % 2..2 {
                        self.keys.push(last_value.clone());
                    }
                }
            }

            NuccAnmKeyFormat::ScaleShortTable => {
                if self.keys.len() % 2 != 0 {
                    let last_value = self.keys.last().unwrap().clone();

                    for _ in self.keys.len() % 2..2 {
                        self.keys.push(last_value.clone());
                    }
                }
            }

            NuccAnmKeyFormat::OpacityShortTableNoInterp => {
                if self.keys.len() % 4 != 0 {
                    let last_value = self.keys.last().unwrap().clone();

                    for _ in self.keys.len() % 4..4 {
                        self.keys.push(last_value.clone());
                    }
                }
            }

            _ => {}
        }
    }
}

fn read_tracks<'it, R, T, Arg, Ret, It>(
    it: It,
) -> impl FnOnce(&mut R, &ReadOptions, ()) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    R: Read + Seek,
    Arg: Clone + 'static,
    Ret: FromIterator<T> + 'static,
    It: Iterator<Item = &'it Arg> + 'it,
{
    move |reader, options, _| {
        it.map(|arg| T::read_options(reader, options, arg.clone()))
            .collect()
    }
}

// compute pad exactly like the 010 template
#[allow(dead_code)]
fn pad_bytes_for(header: &TrackHeader) -> i64 {
    match header.key_format {
        // 0x0F and 0x10: if odd number of frames, add 2 bytes
        NuccAnmKeyFormat::OpacityShortTable | NuccAnmKeyFormat::ScaleShortTable => {
            if header.frame_count % 2 != 0 {
                2
            } else {
                0
            }
        }
        // 0x14: RGB — add (frame_count % 4) bytes
        NuccAnmKeyFormat::ColorRGBTable => (header.frame_count % 4) as i64,
        // 0x1D: if (frame_count % 4) != 0, add 2 bytes
        NuccAnmKeyFormat::OpacityShortTableNoInterp => {
            if header.frame_count % 4 != 0 {
                2
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn read_track_data<R: Read + Seek>(
    reader: &mut R,
    _opts: &ReadOptions,
    header: TrackHeader,
) -> BinResult<Vec<NuccAnmKey>> {
    use binrw::BinReaderExt;
    use std::io::{Cursor, SeekFrom};

    let bytes_per_frame = header.key_format.size_per_frame();
    let total_bytes = bytes_per_frame * header.frame_count as usize;

    let mut buf = vec![0u8; total_bytes];
    reader.read_exact(&mut buf)?;
    let mut sub = Cursor::new(&buf);

    let mut keyframes = Vec::with_capacity(header.frame_count as usize);

    for _ in 0..header.frame_count {
        match header.key_format {
            NuccAnmKeyFormat::Vector3Fixed
            | NuccAnmKeyFormat::Vector3Table
            | NuccAnmKeyFormat::Vector3TableNoInterp
            | NuccAnmKeyFormat::EulerXYZFixed => {
                let x = sub.read_be::<f32>()?;
                let y = sub.read_be::<f32>()?;
                let z = sub.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Vec3 { values: (x, y, z) });
            }

            NuccAnmKeyFormat::Vector3Linear => {
                let frame = sub.read_be::<i32>()?;
                let x = sub.read_be::<f32>()?;
                let y = sub.read_be::<f32>()?;
                let z = sub.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Vec3Linear {
                    frame,
                    values: (x, y, z),
                });
            }

            NuccAnmKeyFormat::QuaternionLinear => {
                let frame = sub.read_be::<i32>()?;
                let x = sub.read_be::<f32>()?;
                let y = sub.read_be::<f32>()?;
                let z = sub.read_be::<f32>()?;
                let w = sub.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Vec4Linear {
                    frame,
                    values: (x, y, z, w),
                });
            }

            NuccAnmKeyFormat::QuaternionShortTable
            | NuccAnmKeyFormat::QuaternionShortTableNoInterp => {
                let x = sub.read_be::<i16>()?;
                let y = sub.read_be::<i16>()?;
                let z = sub.read_be::<i16>()?;
                let w = sub.read_be::<i16>()?;
                keyframes.push(NuccAnmKey::ShortVec4 {
                    values: (x, y, z, w),
                });
            }

            NuccAnmKeyFormat::FloatFixed
            | NuccAnmKeyFormat::FloatTable
            | NuccAnmKeyFormat::FloatTableNoInterp => {
                let v = sub.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::Float { values: v });
            }

            NuccAnmKeyFormat::FloatLinear => {
                let frame = sub.read_be::<i32>()?;
                let v = sub.read_be::<f32>()?;
                keyframes.push(NuccAnmKey::FloatLinear { frame, values: v });
            }

            NuccAnmKeyFormat::OpacityShortTable | NuccAnmKeyFormat::OpacityShortTableNoInterp => {
                let v = sub.read_be::<i16>()?;
                keyframes.push(NuccAnmKey::I16Vec { values: v });
            }

            NuccAnmKeyFormat::ScaleShortTable => {
                let x = sub.read_be::<i16>()?;
                let y = sub.read_be::<i16>()?;
                let z = sub.read_be::<i16>()?;
                keyframes.push(NuccAnmKey::I16Vec3 { values: (x, y, z) });
            }

            NuccAnmKeyFormat::ColorRGBTable => {
                let r = sub.read_be::<u8>()?;
                let g = sub.read_be::<u8>()?;
                let b = sub.read_be::<u8>()?;
                keyframes.push(NuccAnmKey::Color { values: (r, g, b) });
            }

            _ => dbg!(println!(
                "Warning: Unknown key format {:?}, filling with default values",
                header.key_format
            )),
        }
    }

    let pad = match header.key_format {
        NuccAnmKeyFormat::OpacityShortTable | NuccAnmKeyFormat::ScaleShortTable => {
            if header.frame_count % 2 != 0 {
                2
            } else {
                0
            }
        }
        NuccAnmKeyFormat::ColorRGBTable => (header.frame_count % 4) as i64,
        NuccAnmKeyFormat::OpacityShortTableNoInterp => {
            if header.frame_count % 4 != 0 {
                2
            } else {
                0
            }
        }
        _ => 0,
    };

    if pad > 0 {
        reader.seek(SeekFrom::Current(pad))?;
    }

    Ok(keyframes)
}

impl NuccChunk for NuccChunkAnm {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAnm
    }

    fn version(&self) -> u16 {
        self.version
    }
}
