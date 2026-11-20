//! Named Binary Tag (NBT) serializer, deserializer, and compact BitArray for Minecraft schematics.

use std::collections::HashMap;
use std::io::{self, Cursor, Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtTag>),
    Compound(HashMap<String, NbtTag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtTag {
    pub fn tag_id(&self) -> u8 {
        match self {
            NbtTag::End => 0,
            NbtTag::Byte(_) => 1,
            NbtTag::Short(_) => 2,
            NbtTag::Int(_) => 3,
            NbtTag::Long(_) => 4,
            NbtTag::Float(_) => 5,
            NbtTag::Double(_) => 6,
            NbtTag::ByteArray(_) => 7,
            NbtTag::String(_) => 8,
            NbtTag::List(_) => 9,
            NbtTag::Compound(_) => 10,
            NbtTag::IntArray(_) => 11,
            NbtTag::LongArray(_) => 12,
        }
    }

    pub fn as_compound(&self) -> Option<&HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_compound_mut(&mut self) -> Option<&mut HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let NbtTag::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            NbtTag::Int(v) => Some(*v),
            NbtTag::Byte(v) => Some(*v as i32),
            NbtTag::Short(v) => Some(*v as i32),
            _ => None,
        }
    }

    pub fn as_long(&self) -> Option<i64> {
        match self {
            NbtTag::Long(v) => Some(*v),
            NbtTag::Int(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<NbtTag>> {
        if let NbtTag::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_long_array(&self) -> Option<&Vec<i64>> {
        if let NbtTag::LongArray(arr) = self {
            Some(arr)
        } else {
            None
        }
    }
}

pub struct NbtReader<R> {
    reader: R,
}

impl<R: Read> NbtReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_root(&mut self) -> io::Result<(String, NbtTag)> {
        let tag_id = self.read_u8()?;
        if tag_id == 0 {
            return Ok((String::new(), NbtTag::End));
        }
        if tag_id != 10 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected root compound (10), got {}", tag_id),
            ));
        }
        let root_name = self.read_string()?;
        let compound = self.read_compound_content()?;
        Ok((root_name, NbtTag::Compound(compound)))
    }

    fn read_tag(&mut self, tag_id: u8) -> io::Result<NbtTag> {
        match tag_id {
            0 => Ok(NbtTag::End),
            1 => Ok(NbtTag::Byte(self.read_i8()?)),
            2 => Ok(NbtTag::Short(self.read_i16()?)),
            3 => Ok(NbtTag::Int(self.read_i32()?)),
            4 => Ok(NbtTag::Long(self.read_i64()?)),
            5 => Ok(NbtTag::Float(self.read_f32()?)),
            6 => Ok(NbtTag::Double(self.read_f64()?)),
            7 => {
                let len = self.read_i32()? as usize;
                let mut buf = vec![0u8; len];
                self.reader.read_exact(&mut buf)?;
                let signed: Vec<i8> = buf.into_iter().map(|b| b as i8).collect();
                Ok(NbtTag::ByteArray(signed))
            }
            8 => Ok(NbtTag::String(self.read_string()?)),
            9 => {
                let elem_type = self.read_u8()?;
                let len = self.read_i32()?;
                if len <= 0 {
                    return Ok(NbtTag::List(Vec::new()));
                }
                let mut list = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    list.push(self.read_tag(elem_type)?);
                }
                Ok(NbtTag::List(list))
            }
            10 => Ok(NbtTag::Compound(self.read_compound_content()?)),
            11 => {
                let len = self.read_i32()? as usize;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(self.read_i32()?);
                }
                Ok(NbtTag::IntArray(arr))
            }
            12 => {
                let len = self.read_i32()? as usize;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(self.read_i64()?);
                }
                Ok(NbtTag::LongArray(arr))
            }
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown NBT tag type: {}", other),
            )),
        }
    }

    fn read_compound_content(&mut self) -> io::Result<HashMap<String, NbtTag>> {
        let mut map = HashMap::new();
        loop {
            let tag_id = self.read_u8()?;
            if tag_id == 0 {
                break;
            }
            let name = self.read_string()?;
            let tag = self.read_tag(tag_id)?;
            map.insert(name, tag);
        }
        Ok(map)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_i8(&mut self) -> io::Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_i16(&mut self) -> io::Result<i16> {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }

    fn read_f64(&mut self) -> io::Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }

    fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_i16()? as usize;
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub struct NbtWriter<W> {
    writer: W,
}

impl<W: Write> NbtWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_root(&mut self, root_name: &str, tag: &NbtTag) -> io::Result<()> {
        self.write_u8(tag.tag_id())?;
        self.write_string(root_name)?;
        self.write_tag_payload(tag)?;
        Ok(())
    }

    pub fn write_tag_payload(&mut self, tag: &NbtTag) -> io::Result<()> {
        match tag {
            NbtTag::End => Ok(()),
            NbtTag::Byte(v) => self.write_i8(*v),
            NbtTag::Short(v) => self.write_i16(*v),
            NbtTag::Int(v) => self.write_i32(*v),
            NbtTag::Long(v) => self.write_i64(*v),
            NbtTag::Float(v) => self.write_f32(*v),
            NbtTag::Double(v) => self.write_f64(*v),
            NbtTag::ByteArray(arr) => {
                self.write_i32(arr.len() as i32)?;
                for &b in arr {
                    self.write_i8(b)?;
                }
                Ok(())
            }
            NbtTag::String(s) => self.write_string(s),
            NbtTag::List(list) => {
                if list.is_empty() {
                    self.write_u8(0)?;
                    self.write_i32(0)?;
                } else {
                    let elem_id = list[0].tag_id();
                    self.write_u8(elem_id)?;
                    self.write_i32(list.len() as i32)?;
                    for item in list {
                        self.write_tag_payload(item)?;
                    }
                }
                Ok(())
            }
            NbtTag::Compound(map) => {
                for (key, val) in map {
                    self.write_u8(val.tag_id())?;
                    self.write_string(key)?;
                    self.write_tag_payload(val)?;
                }
                self.write_u8(0)?; // End tag
                Ok(())
            }
            NbtTag::IntArray(arr) => {
                self.write_i32(arr.len() as i32)?;
                for &i in arr {
                    self.write_i32(i)?;
                }
                Ok(())
            }
            NbtTag::LongArray(arr) => {
                self.write_i32(arr.len() as i32)?;
                for &l in arr {
                    self.write_i64(l)?;
                }
                Ok(())
            }
        }
    }

    fn write_u8(&mut self, val: u8) -> io::Result<()> {
        self.writer.write_all(&[val])
    }

    fn write_i8(&mut self, val: i8) -> io::Result<()> {
        self.write_u8(val as u8)
    }

    fn write_i16(&mut self, val: i16) -> io::Result<()> {
        self.writer.write_all(&val.to_be_bytes())
    }

    fn write_i32(&mut self, val: i32) -> io::Result<()> {
        self.writer.write_all(&val.to_be_bytes())
    }

    fn write_i64(&mut self, val: i64) -> io::Result<()> {
        self.writer.write_all(&val.to_be_bytes())
    }

    fn write_f32(&mut self, val: f32) -> io::Result<()> {
        self.writer.write_all(&val.to_be_bytes())
    }

    fn write_f64(&mut self, val: f64) -> io::Result<()> {
        self.writer.write_all(&val.to_be_bytes())
    }

    fn write_string(&mut self, val: &str) -> io::Result<()> {
        let bytes = val.as_bytes();
        self.write_i16(bytes.len() as i16)?;
        self.writer.write_all(bytes)
    }
}

/// Compact BitArray used by Minecraft & Litematica for packing palette indices into long arrays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitArray {
    bits_per_entry: usize,
    capacity: usize,
    max_entry_value: u64,
    data: Vec<i64>,
}

impl BitArray {
    pub fn new(bits_per_entry: usize, capacity: usize) -> Self {
        let bits_per_entry = bits_per_entry.max(1);
        let entries_per_long = 64 / bits_per_entry;
        let num_longs = (capacity + entries_per_long - 1) / entries_per_long;
        let max_entry_value = (1u64 << bits_per_entry) - 1;
        Self {
            bits_per_entry,
            capacity,
            max_entry_value,
            data: vec![0; num_longs],
        }
    }

    pub fn from_data(bits_per_entry: usize, capacity: usize, data: Vec<i64>) -> Self {
        let bits_per_entry = bits_per_entry.max(1);
        let max_entry_value = (1u64 << bits_per_entry) - 1;
        Self {
            bits_per_entry,
            capacity,
            max_entry_value,
            data,
        }
    }

    pub fn get(&self, index: usize) -> u32 {
        if index >= self.capacity {
            return 0;
        }
        let entries_per_long = 64 / self.bits_per_entry;
        let long_index = index / entries_per_long;
        if long_index >= self.data.len() {
            return 0;
        }
        let bit_offset = (index % entries_per_long) * self.bits_per_entry;
        let raw_long = self.data[long_index] as u64;
        ((raw_long >> bit_offset) & self.max_entry_value) as u32
    }

    pub fn set(&mut self, index: usize, value: u32) {
        if index >= self.capacity {
            return;
        }
        let entries_per_long = 64 / self.bits_per_entry;
        let long_index = index / entries_per_long;
        if long_index >= self.data.len() {
            return;
        }
        let bit_offset = (index % entries_per_long) * self.bits_per_entry;
        let val_masked = (value as u64) & self.max_entry_value;
        let mask = !(self.max_entry_value << bit_offset);
        let raw = self.data[long_index] as u64;
        self.data[long_index] = ((raw & mask) | (val_masked << bit_offset)) as i64;
    }

    pub fn data(&self) -> &[i64] {
        &self.data
    }

    pub fn bits_per_entry(&self) -> usize {
        self.bits_per_entry
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Fast decompressor supporting uncompressed, gzip, and zlib NBT payloads.
pub fn parse_nbt_buffer(buffer: &[u8]) -> io::Result<(String, NbtTag)> {
    if buffer.len() >= 2 && buffer[0] == 0x1F && buffer[1] == 0x8B {
        // Gzip compressed payload
        let decompressed = decompress_gzip(buffer)?;
        let mut cursor = Cursor::new(decompressed);
        let mut reader = NbtReader::new(&mut cursor);
        reader.read_root()
    } else if buffer.len() >= 2 && buffer[0] == 0x78 {
        // Zlib compressed payload
        let decompressed = decompress_zlib(buffer)?;
        let mut cursor = Cursor::new(decompressed);
        let mut reader = NbtReader::new(&mut cursor);
        reader.read_root()
    } else {
        // Uncompressed NBT
        let mut cursor = Cursor::new(buffer);
        let mut reader = NbtReader::new(&mut cursor);
        reader.read_root()
    }
}

pub fn serialize_nbt_buffer(root_name: &str, tag: &NbtTag) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut writer = NbtWriter::new(&mut out);
    writer.write_root(root_name, tag)?;
    Ok(out)
}

/// Fallback decompression helper for standard gzip/zlib streams
fn decompress_gzip(input: &[u8]) -> io::Result<Vec<u8>> {
    // If the stream contains standard gzip, locate the raw deflate stream after header
    if input.len() < 10 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Gzip header too short"));
    }
    let flg = input[3];
    let mut offset = 10;
    if (flg & 0x04) != 0 && input.len() >= offset + 2 {
        let xlen = u16::from_le_bytes([input[offset], input[offset + 1]]) as usize;
        offset += 2 + xlen;
    }
    if (flg & 0x08) != 0 {
        while offset < input.len() && input[offset] != 0 {
            offset += 1;
        }
        offset += 1; // null terminator
    }
    if (flg & 0x10) != 0 {
        while offset < input.len() && input[offset] != 0 {
            offset += 1;
        }
        offset += 1; // null terminator
    }
    if (flg & 0x02) != 0 {
        offset += 2; // header crc
    }
    let raw_len = if input.len() > offset + 8 { input.len() - 8 } else { input.len() };
    inflate_raw(&input[offset..raw_len])
}

fn decompress_zlib(input: &[u8]) -> io::Result<Vec<u8>> {
    if input.len() < 6 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Zlib header too short"));
    }
    let raw_len = if input.len() > 4 { input.len() - 4 } else { input.len() };
    inflate_raw(&input[2..raw_len])
}

/// Minimal inflate for uncompressed (type 0) or raw blocks
fn inflate_raw(input: &[u8]) -> io::Result<Vec<u8>> {
    // If stored uncompressed in deflate format or direct pass-through
    let mut out = Vec::new();
    let mut cursor = 0;
    while cursor < input.len() {
        if cursor + 5 <= input.len() && (input[cursor] & 0x07) == 0 {
            // Uncompressed Deflate block
            let len = u16::from_le_bytes([input[cursor + 1], input[cursor + 2]]) as usize;
            cursor += 5;
            if cursor + len <= input.len() {
                out.extend_from_slice(&input[cursor..cursor + len]);
                cursor += len;
                continue;
            }
        }
        // Direct stream copy
        out.extend_from_slice(&input[cursor..]);
        break;
    }
    Ok(out)
}
