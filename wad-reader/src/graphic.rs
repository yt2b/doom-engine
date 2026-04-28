use crate::read::{read_i16, read_u16, read_u32};
use anyhow::Result;
use std::collections::HashMap;

pub struct Graphic {
    pub palettes: Vec<Vec<(u8, u8, u8)>>,
    pub sprites: HashMap<String, Patch>,
}

impl Graphic {
    pub fn new(palettes: Vec<Vec<(u8, u8, u8)>>, sprites: HashMap<String, Patch>) -> Self {
        Self { palettes, sprites }
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
