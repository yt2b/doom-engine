use crate::core::map::Map;
use anyhow::Result;
use std::{
    fs::File,
    io::{BufReader, Read},
};

pub struct Wad {
    pub ident: String,
    pub lumps: Vec<Lump>,
}

impl Wad {
    pub fn new(file: File) -> Result<Self> {
        // WADファイル全体を読み込む
        let mut reader = BufReader::new(file);
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        // ヘッダーを読み込む
        let ident = read_string(&buf, 0, 4)?;
        let num_lumps = read_i32(&buf, 4)? as usize;
        let info_table_ofs = read_i32(&buf, 8)? as usize;
        let mut lumps = vec![];
        // Lumpを読み込む
        for i in 0..num_lumps {
            let ofs = info_table_ofs + 16 * i;
            let file_pos = read_i32(&buf, ofs)? as usize;
            let size = read_i32(&buf, ofs + 4)? as usize;
            let name = read_string(&buf, ofs + 8, 8)?;
            let data = buf[file_pos..file_pos + size].to_vec();
            lumps.push(Lump::new(name, data));
        }
        Ok(Self { ident, lumps })
    }

    pub fn get_lump(&self, name: &str) -> Result<&Lump> {
        self.lumps
            .iter()
            .find(|lump| lump.name == name)
            .ok_or_else(|| anyhow::anyhow!("Lump named '{}' not found", name))
    }

    pub fn get_lump_index(&self, name: &str) -> Result<usize> {
        self.lumps
            .iter()
            .position(|lump| lump.name == name)
            .ok_or_else(|| anyhow::anyhow!("Lump named '{}' not found", name))
    }

    pub fn read_map(&self, name: &str) -> Result<Map> {
        let start_index = self.get_lump_index(name)?;
        let map_lumps = &self.lumps[start_index..];
        Map::new_from_lumps(map_lumps)
    }
}

pub struct Lump {
    pub name: String,
    pub bytes: Vec<u8>,
}

impl Lump {
    pub fn new(name: String, bytes: Vec<u8>) -> Self {
        Self { name, bytes }
    }
}

pub fn read_string(data: &[u8], offset: usize, length: usize) -> Result<String> {
    Ok(String::from_utf8(data[offset..offset + length].to_vec())?
        .trim_end_matches('\0')
        .to_string())
}

pub fn read_i16(data: &[u8], offset: usize) -> Result<i16> {
    Ok(i16::from_le_bytes(data[offset..offset + 2].try_into()?))
}

pub fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(data[offset..offset + 2].try_into()?))
}

pub fn read_i32(data: &[u8], offset: usize) -> Result<i32> {
    Ok(i32::from_le_bytes(data[offset..offset + 4].try_into()?))
}

pub fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(data[offset..offset + 4].try_into()?))
}
