//! Byte order used by serialized NUCC data.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NuccEndianType {
    #[default]
    Big,
    Little,
}
