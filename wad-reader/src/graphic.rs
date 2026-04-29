use crate::{
    Wad,
    read::{read_i16, read_i32, read_string, read_u16, read_u32},
};
use anyhow::Result;
use std::collections::HashMap;

pub struct Graphic {
    pub palettes: Vec<Vec<(u8, u8, u8)>>,
    pub sprites: HashMap<String, Patch>,
    pub patch_names: Vec<String>,
    pub texture_patches: HashMap<String, Patch>,
    pub textures: Vec<Texture>,
}

impl Graphic {
    pub fn new(
        palettes: Vec<Vec<(u8, u8, u8)>>,
        sprites: HashMap<String, Patch>,
        patch_names: Vec<String>,
        texture_patches: HashMap<String, Patch>,
        textures: Vec<Texture>,
    ) -> Self {
        Self {
            palettes,
            sprites,
            patch_names,
            texture_patches,
            textures,
        }
    }
}

pub fn get_palettes(bytes: &[u8]) -> Vec<Vec<(u8, u8, u8)>> {
    // 768バイトごとに区切って、さらに3バイトごとにRGB値を読み取る
    bytes
        .chunks(768)
        .map(|chunk| {
            chunk
                .chunks(3)
                .map(|rgb| (rgb[0], rgb[1], rgb[2]))
                .collect()
        })
        .collect()
}

pub struct Patch {
    pub width: u16,
    pub height: u16,
    pub left_offset: i16,
    pub top_offset: i16,
    pub patch_columns: Vec<PatchColumn>,
}

impl Patch {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self> {
        let width = read_u16(bytes, 0)?;
        // PatchColumnを読み込む
        let mut patch_columns = Vec::new();
        for i in 0..width {
            let offset = read_u32(bytes, 8 + i as usize * 4)? as usize;
            let patch_column = PatchColumn::new_from_bytes(&bytes[offset..])?;
            patch_columns.push(patch_column);
        }
        Ok(Self {
            width,
            height: read_u16(bytes, 2)?,
            left_offset: read_i16(bytes, 4)?,
            top_offset: read_i16(bytes, 6)?,
            patch_columns,
        })
    }
}

// (y座標のオフセット, ピクセルデータ)のタプルリスト
pub struct PatchColumn(pub Vec<(u8, Vec<u8>)>);

impl PatchColumn {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut columns = Vec::new();
        let mut offset = 0;
        loop {
            let top_delta = bytes[offset];
            if top_delta == 255 {
                break;
            }
            let length = bytes[offset + 1] as usize;
            let data = &bytes[offset + 3..offset + 3 + length];
            columns.push((top_delta, data.to_vec()));
            offset += 4 + length;
        }
        Ok(Self(columns))
    }
}

pub fn create_textures(wad: &Wad, name: &str) -> Result<Vec<Texture>> {
    let texture = wad.get_lump(name)?;
    let num_textures = read_i32(&texture.bytes, 0)? as usize;
    let offsets = (0..num_textures)
        .map(|i| read_i32(&texture.bytes, 4 + i * 4))
        .collect::<Result<Vec<i32>>>()?;
    let mut textures = Vec::new();
    for offset in offsets {
        let texture_offset = offset as usize;
        let name = read_string(&texture.bytes, texture_offset, 8)?;
        let width = read_u16(&texture.bytes, texture_offset + 12)?;
        let height = read_u16(&texture.bytes, texture_offset + 14)?;
        let num_patches = read_i16(&texture.bytes, texture_offset + 20)? as usize;
        let mut patches = Vec::new();
        for i in 0..num_patches {
            let patch_offset = texture_offset + 22 + i * 10;
            let x_offset = read_i16(&texture.bytes, patch_offset)?;
            let y_offset = read_i16(&texture.bytes, patch_offset + 2)?;
            let idx = read_i16(&texture.bytes, patch_offset + 4)? as usize;
            patches.push(TexturePatch::new(x_offset, y_offset, idx));
        }
        textures.push(Texture::new(name, width, height, patches));
    }
    Ok(textures)
}

pub struct Texture {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub patches: Vec<TexturePatch>,
}

impl Texture {
    pub fn new(name: String, width: u16, height: u16, patches: Vec<TexturePatch>) -> Self {
        Self {
            name,
            width,
            height,
            patches,
        }
    }
}

pub struct TexturePatch {
    pub offset_x: i16,
    pub offset_y: i16,
    pub idx: usize,
}

impl TexturePatch {
    pub fn new(offset_x: i16, offset_y: i16, idx: usize) -> Self {
        Self {
            offset_x,
            offset_y,
            idx,
        }
    }
}
