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
    graphic::{Graphic, Patch},
    map::Map,
};
use crate::{
    graphic::{create_textures, get_palettes},
    read::{read_i32, read_string},
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

    fn get_lump(&self, name: &str) -> Result<&Lump> {
        self.lumps
            .iter()
            .find(|lump| lump.name == name)
            .ok_or_else(|| anyhow::anyhow!("Lump named '{}' not found", name))
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
        // パッチ名を読み込む
        let pnames = self.get_lump("PNAMES")?;
        let num_patches = read_i32(&pnames.bytes, 0)? as usize;
        let mut patch_names = Vec::new();
        for i in 0..num_patches {
            let name = read_string(&pnames.bytes, 4 + i * 8, 8)?;
            patch_names.push(name.to_uppercase());
        }
        // テクスチャ用のパッチを読み込む
        let mut texture_patches = HashMap::new();
        for name in &patch_names {
            if let Ok(lump) = self.get_lump(name) {
                texture_patches.insert(name.clone(), Patch::new_from_bytes(&lump.bytes)?);
            }
        }
        // テクスチャを読み込む
        let mut textures = create_textures(self, "TEXTURE1")?;
        if self.get_lump("TEXTURE2").is_ok() {
            let mut textures2 = create_textures(self, "TEXTURE2")?;
            textures.append(&mut textures2);
        }
        // Flatを読み込む
        let start_idx = self.get_lump_index("F_START")?;
        let end_idx = self.get_lump_index("F_END")?;
        let mut flats = HashMap::new();
        for lump in &self.lumps[start_idx + 1..end_idx] {
            if lump.bytes.len() != 64 * 64 {
                continue;
            }
            flats.insert(lump.name.clone(), lump.bytes.clone());
        }
        Ok(Graphic::new(
            pallets,
            sprites,
            patch_names,
            texture_patches,
            textures,
            flats,
        ))
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
