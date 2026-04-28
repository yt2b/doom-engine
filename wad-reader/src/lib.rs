use anyhow::Result;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
};

pub mod graphic;
pub mod map;
mod read;

use crate::{
    graphic::get_palettes,
    read::{read_i32, read_string},
};
use crate::{
    graphic::{Graphic, Patch},
    map::Map,
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

    fn get_lump_index(&self, name: &str) -> Result<usize> {
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

    pub fn read_graphic(&self) -> Result<Graphic> {
        // パレットを読み込む
        let lump = self
            .lumps
            .iter()
            .find(|lump| lump.name == "PLAYPAL")
            .ok_or_else(|| anyhow::anyhow!("Pallet named 'PLAYPAL' not found"))?;
        let pallets = get_palettes(&lump.bytes);
        // スプライトを読み込む
        let mut sprites = HashMap::new();
        let start_idx = self.get_lump_index("S_START")?;
        let end_idx = self.get_lump_index("S_END")?;
        for lump in &self.lumps[start_idx + 1..end_idx] {
            sprites.insert(lump.name.clone(), Patch::new_from_bytes(&lump.bytes)?);
        }
        Ok(Graphic::new(pallets, sprites))
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
