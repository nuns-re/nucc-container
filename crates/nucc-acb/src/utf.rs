use std::collections::HashMap;
use std::sync::Arc;

use nucc_core::{NuccError, Result};

const HEADER_SIZE: usize = 0x20;
const COLUMN_FLAG_NAME: u8 = 0x10;
const COLUMN_FLAG_DEFAULT: u8 = 0x20;
const COLUMN_FLAG_ROW: u8 = 0x40;
const COLUMN_FLAG_UNDEFINED: u8 = 0x80;

#[derive(Debug, Clone, PartialEq)]
pub enum UtfValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Data { offset: usize, size: usize },
    U128([u8; 16]),
}

impl UtfValue {
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Self::U8(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            Self::U16(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::U32(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::U64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Self::F32(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_data(&self) -> Option<(usize, usize)> {
        match self {
            Self::Data { offset, size } => Some((*offset, *size)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UtfTable {
    data: Arc<[u8]>,
    offset: usize,
    table_size: usize,
    version: u16,
    rows: usize,
    row_width: usize,
    rows_offset: usize,
    strings_offset: usize,
    data_offset: usize,
    name: String,
    columns: Vec<UtfColumn>,
    column_map: HashMap<String, usize>,
    values: Vec<Vec<UtfValue>>,
}

#[derive(Debug, Clone)]
struct UtfColumn {
    name: String,
    kind: UtfType,
    source: ColumnSource,
}

#[derive(Debug, Clone, Copy)]
enum ColumnSource {
    Default(usize),
    Row(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtfType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
    Data,
    U128,
}

impl UtfType {
    fn from_code(code: u8) -> Result<Self> {
        Ok(match code {
            0x00 => Self::U8,
            0x01 => Self::I8,
            0x02 => Self::U16,
            0x03 => Self::I16,
            0x04 => Self::U32,
            0x05 => Self::I32,
            0x06 => Self::U64,
            0x07 => Self::I64,
            0x08 => Self::F32,
            0x09 => Self::F64,
            0x0a => Self::String,
            0x0b => Self::Data,
            0x0c => Self::U128,
            _ => return Err(NuccError::Unsupported(format!("UTF column type {code:#x}"))),
        })
    }

    fn size(self) -> usize {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::F32 | Self::String => 4,
            Self::U64 | Self::I64 | Self::F64 | Self::Data => 8,
            Self::U128 => 16,
        }
    }
}

impl UtfTable {
    pub fn parse(data: &[u8]) -> Result<Self> {
        Self::parse_arc(Arc::from(data), 0)
    }

    pub fn parse_arc(data: Arc<[u8]>, offset: usize) -> Result<Self> {
        let table_size = add(read_u32be(&data, add(offset, 0x04)?)? as usize, 8)?;
        let table_end = add(offset, table_size)?;
        if table_end > data.len() {
            return Err(NuccError::Truncated);
        }
        if &data[offset..offset + 4] != b"@UTF" {
            return Err(NuccError::InvalidHeader);
        }

        let version = read_u16be(&data, offset + 0x08)?;
        if version > 1 {
            return Err(NuccError::Unsupported(format!("UTF version {version}")));
        }

        let rows_offset = add(read_u16be(&data, offset + 0x0a)? as usize, 8)?;
        let strings_offset = add(read_u32be(&data, offset + 0x0c)? as usize, 8)?;
        let data_offset = add(read_u32be(&data, offset + 0x10)? as usize, 8)?;
        let name_offset = read_u32be(&data, offset + 0x14)? as usize;
        let columns = read_u16be(&data, offset + 0x18)? as usize;
        let row_width = read_u16be(&data, offset + 0x1a)? as usize;
        let rows = read_u32be(&data, offset + 0x1c)? as usize;

        if columns == 0 {
            return Err(NuccError::InvalidIndex("UTF table has no columns".into()));
        }
        if rows_offset < HEADER_SIZE
            || rows_offset > strings_offset
            || strings_offset > data_offset
            || data_offset > table_size
        {
            return Err(NuccError::InvalidIndex(
                "invalid UTF section offsets".into(),
            ));
        }

        let schema_size = rows_offset - HEADER_SIZE;
        let rows_size = strings_offset - rows_offset;
        let strings_size = data_offset - strings_offset;
        let data_size = table_size - data_offset;
        if strings_size == 0 || name_offset >= strings_size {
            return Err(NuccError::InvalidIndex("invalid UTF string section".into()));
        }
        if rows_size < rows.checked_mul(row_width).ok_or(NuccError::Overflow)? {
            return Err(NuccError::InvalidIndex("UTF rows exceed section".into()));
        }

        let schema_start = add(offset, HEADER_SIZE)?;
        let strings_start = add(offset, strings_offset)?;
        let strings_end = add(strings_start, strings_size)?;
        let data_start = add(offset, data_offset)?;
        let sections = UtfSections {
            strings_start,
            strings_end,
            data_start,
            data_size,
        };
        let name = read_cstr(&data, add(strings_start, name_offset)?, strings_end)?.to_owned();

        let mut schema = Vec::with_capacity(columns);
        let mut schema_cursor = 0;
        let mut row_cursor = 0;
        for _ in 0..columns {
            if add(schema_cursor, 5)? > schema_size {
                return Err(NuccError::InvalidIndex("UTF schema is truncated".into()));
            }

            let info = read_u8(&data, add(schema_start, schema_cursor)?)?;
            let column_name_offset =
                read_u32be(&data, add(schema_start, add(schema_cursor, 1)?)?)? as usize;
            schema_cursor = add(schema_cursor, 5)?;

            let flags = info & 0xf0;
            let kind = UtfType::from_code(info & 0x0f)?;
            if flags & COLUMN_FLAG_NAME == 0 || flags & COLUMN_FLAG_UNDEFINED != 0 {
                return Err(NuccError::Unsupported(format!(
                    "UTF column flags {flags:#x}"
                )));
            }

            let column_name =
                read_cstr(&data, add(strings_start, column_name_offset)?, strings_end)?.to_owned();

            let source = if flags & COLUMN_FLAG_DEFAULT != 0 {
                let source = ColumnSource::Default(schema_cursor);
                schema_cursor = add(schema_cursor, kind.size())?;
                if schema_cursor > schema_size {
                    return Err(NuccError::InvalidIndex(
                        "UTF default value exceeds schema".into(),
                    ));
                }
                source
            } else if flags & COLUMN_FLAG_ROW != 0 {
                let source = ColumnSource::Row(row_cursor);
                row_cursor = add(row_cursor, kind.size())?;
                if row_cursor > rows_size {
                    return Err(NuccError::InvalidIndex("UTF row value exceeds row".into()));
                }
                source
            } else {
                return Err(NuccError::Unsupported(format!(
                    "UTF column source {flags:#x}"
                )));
            };

            schema.push(UtfColumn {
                name: column_name,
                kind,
                source,
            });
        }

        let mut column_map = HashMap::with_capacity(schema.len());
        for (index, column) in schema.iter().enumerate() {
            column_map.insert(column.name.clone(), index);
        }

        let mut values = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut row_values = Vec::with_capacity(schema.len());
            for column in &schema {
                let (position, limit) = match column.source {
                    ColumnSource::Default(position) => (
                        add(schema_start, position)?,
                        add(schema_start, schema_size)?,
                    ),
                    ColumnSource::Row(position) => {
                        let row_start = add(
                            offset,
                            add(
                                rows_offset,
                                row.checked_mul(row_width).ok_or(NuccError::Overflow)?,
                            )?,
                        )?;
                        (
                            add(row_start, position)?,
                            add(add(offset, rows_offset)?, rows_size)?,
                        )
                    }
                };
                row_values.push(read_value(&data, position, limit, column.kind, sections)?);
            }
            values.push(row_values);
        }

        Ok(Self {
            data,
            offset,
            table_size,
            version,
            rows,
            row_width,
            rows_offset,
            strings_offset,
            data_offset,
            name,
            columns: schema,
            column_map,
            values,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn table_size(&self) -> usize {
        self.table_size
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn row_count(&self) -> usize {
        self.rows
    }

    pub fn row_width(&self) -> usize {
        self.row_width
    }

    pub fn rows_offset(&self) -> usize {
        self.rows_offset
    }

    pub fn strings_offset(&self) -> usize {
        self.strings_offset
    }

    pub fn data_offset(&self) -> usize {
        self.data_offset
    }

    pub fn columns(&self) -> impl Iterator<Item = &str> {
        self.columns.iter().map(|column| column.name.as_str())
    }

    pub fn get(&self, row: usize, name: &str) -> Option<&UtfValue> {
        let column = *self.column_map.get(name)?;
        self.values.get(row)?.get(column)
    }

    pub fn get_u8(&self, row: usize, name: &str) -> Option<u8> {
        self.get(row, name)?.as_u8()
    }

    pub fn get_u16(&self, row: usize, name: &str) -> Option<u16> {
        self.get(row, name)?.as_u16()
    }

    pub fn get_u32(&self, row: usize, name: &str) -> Option<u32> {
        self.get(row, name)?.as_u32()
    }

    pub fn get_u64(&self, row: usize, name: &str) -> Option<u64> {
        self.get(row, name)?.as_u64()
    }

    pub fn get_f32(&self, row: usize, name: &str) -> Option<f32> {
        self.get(row, name)?.as_f32()
    }

    pub fn get_string(&self, row: usize, name: &str) -> Option<&str> {
        self.get(row, name)?.as_str()
    }

    pub fn get_data(&self, row: usize, name: &str) -> Option<&[u8]> {
        let (offset, size) = self.get(row, name)?.as_data()?;
        self.data.get(offset..offset.checked_add(size)?)
    }

    pub fn data(&self, value: &UtfValue) -> Option<&[u8]> {
        let (offset, size) = value.as_data()?;
        self.data.get(offset..offset.checked_add(size)?)
    }

    pub fn open_subtable(&self, row: usize, name: &str) -> Result<Self> {
        let value = self
            .get(row, name)
            .ok_or_else(|| NuccError::InvalidIndex(format!("missing UTF column {name}")))?;
        let (offset, size) = value
            .as_data()
            .ok_or_else(|| NuccError::InvalidIndex(format!("UTF column {name} is not data")))?;
        if size == 0 {
            return Err(NuccError::InvalidChunk(format!(
                "UTF column {name} is empty"
            )));
        }
        Self::parse_arc(self.data.clone(), offset)
    }
}

#[derive(Clone, Copy)]
struct UtfSections {
    strings_start: usize,
    strings_end: usize,
    data_start: usize,
    data_size: usize,
}

fn read_value(
    data: &[u8],
    offset: usize,
    limit: usize,
    kind: UtfType,
    sections: UtfSections,
) -> Result<UtfValue> {
    if add(offset, kind.size())? > limit {
        return Err(NuccError::Truncated);
    }

    Ok(match kind {
        UtfType::U8 => UtfValue::U8(read_u8(data, offset)?),
        UtfType::I8 => UtfValue::I8(read_u8(data, offset)? as i8),
        UtfType::U16 => UtfValue::U16(read_u16be(data, offset)?),
        UtfType::I16 => UtfValue::I16(read_u16be(data, offset)? as i16),
        UtfType::U32 => UtfValue::U32(read_u32be(data, offset)?),
        UtfType::I32 => UtfValue::I32(read_u32be(data, offset)? as i32),
        UtfType::U64 => UtfValue::U64(read_u64be(data, offset)?),
        UtfType::I64 => UtfValue::I64(read_u64be(data, offset)? as i64),
        UtfType::F32 => UtfValue::F32(f32::from_bits(read_u32be(data, offset)?)),
        UtfType::F64 => UtfValue::F64(f64::from_bits(read_u64be(data, offset)?)),
        UtfType::String => {
            let string_offset = read_u32be(data, offset)? as usize;
            let value = read_cstr(
                data,
                add(sections.strings_start, string_offset)?,
                sections.strings_end,
            )?;
            UtfValue::String(value.to_owned())
        }
        UtfType::Data => {
            let relative_offset = read_u32be(data, offset)? as usize;
            let size = read_u32be(data, add(offset, 4)?)? as usize;
            if relative_offset
                .checked_add(size)
                .ok_or(NuccError::Overflow)?
                > sections.data_size
            {
                return Err(NuccError::InvalidIndex("UTF data exceeds section".into()));
            }
            UtfValue::Data {
                offset: add(sections.data_start, relative_offset)?,
                size,
            }
        }
        UtfType::U128 => {
            let bytes = data
                .get(offset..add(offset, 16)?)
                .ok_or(NuccError::Truncated)?;
            let mut value = [0u8; 16];
            value.copy_from_slice(bytes);
            UtfValue::U128(value)
        }
    })
}

fn read_cstr(data: &[u8], offset: usize, end: usize) -> Result<&str> {
    if offset >= end || end > data.len() {
        return Err(NuccError::InvalidIndex("invalid UTF string offset".into()));
    }
    let bytes = &data[offset..end];
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    Ok(std::str::from_utf8(&bytes[..length]).unwrap_or(""))
}

fn read_u8(data: &[u8], offset: usize) -> Result<u8> {
    data.get(offset).copied().ok_or(NuccError::Truncated)
}

fn read_u16be(data: &[u8], offset: usize) -> Result<u16> {
    let bytes = data.get(offset..offset + 2).ok_or(NuccError::Truncated)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32be(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data.get(offset..offset + 4).ok_or(NuccError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u64be(data: &[u8], offset: usize) -> Result<u64> {
    let bytes = data.get(offset..offset + 8).ok_or(NuccError::Truncated)?;
    Ok(u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

fn add(left: usize, right: usize) -> Result<usize> {
    left.checked_add(right).ok_or(NuccError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_row_values() {
        let data = minimal_table();
        let table = UtfTable::parse(&data).unwrap();

        assert_eq!(table.name(), "Header");
        assert_eq!(table.row_count(), 1);
        assert_eq!(table.get_u16(0, "CueIndex"), Some(0));
        assert_eq!(table.get_string(0, "CueName"), Some("Test"));
    }

    fn minimal_table() -> Vec<u8> {
        let mut data = vec![0u8; 0x4d];

        data[0x00..0x04].copy_from_slice(b"@UTF");
        data[0x04..0x08].copy_from_slice(&0x45u32.to_be_bytes());
        data[0x08..0x0a].copy_from_slice(&1u16.to_be_bytes());
        data[0x0a..0x0c].copy_from_slice(&0x22u16.to_be_bytes());
        data[0x0c..0x10].copy_from_slice(&0x28u32.to_be_bytes());
        data[0x10..0x14].copy_from_slice(&0x45u32.to_be_bytes());
        data[0x14..0x18].copy_from_slice(&0u32.to_be_bytes());
        data[0x18..0x1a].copy_from_slice(&2u16.to_be_bytes());
        data[0x1a..0x1c].copy_from_slice(&6u16.to_be_bytes());
        data[0x1c..0x20].copy_from_slice(&1u32.to_be_bytes());

        data[0x20] = 0x52;
        data[0x21..0x25].copy_from_slice(&7u32.to_be_bytes());
        data[0x25] = 0x5a;
        data[0x26..0x2a].copy_from_slice(&16u32.to_be_bytes());

        data[0x2a..0x2c].copy_from_slice(&0u16.to_be_bytes());
        data[0x2c..0x30].copy_from_slice(&24u32.to_be_bytes());

        data[0x30..0x36].copy_from_slice(b"Header");
        data[0x36] = 0;
        data[0x37..0x3f].copy_from_slice(b"CueIndex");
        data[0x3f] = 0;
        data[0x40..0x47].copy_from_slice(b"CueName");
        data[0x47] = 0;
        data[0x48..0x4c].copy_from_slice(b"Test");
        data[0x4c] = 0;

        data
    }
}
