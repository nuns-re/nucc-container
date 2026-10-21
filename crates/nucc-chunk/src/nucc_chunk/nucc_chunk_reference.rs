use std::io::{Result, Write};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NuccChunkReference {
    pub indices: [u32; 2],
}

impl NuccChunkReference {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        Some(Self {
            indices: [
                u32::from_be_bytes(data[0..4].try_into().ok()?),
                u32::from_be_bytes(data[4..8].try_into().ok()?),
            ],
        })
    }

    pub fn write_to(&self, output: &mut impl Write) -> Result<()> {
        for index in self.indices {
            output.write_all(&index.to_be_bytes())?;
        }
        Ok(())
    }
}
