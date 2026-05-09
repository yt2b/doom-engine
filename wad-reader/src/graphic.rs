use crate::{
    Wad,
    read::{read_i16, read_i32, read_string, read_u16, read_u32},
};
use anyhow::Result;
use std::collections::HashMap;

pub const FLAT_SIZE: usize = 64;

pub struct Graphic {
    pub palettes: Vec<Vec<(u8, u8, u8)>>,
    pub sprites: HashMap<String, Patch>,
    pub patch_names: Vec<String>,
    pub texture_patches: HashMap<String, Patch>,
    pub textures: HashMap<String, Texture>,
    pub flats: HashMap<String, Vec<usize>>,
}

impl Graphic {
    pub fn new_from_wad(wad: &Wad) -> Result<Self> {
        // パレットを読み込む
        let lump = wad.get_lump("PLAYPAL")?;
        let palettes = get_palettes(&lump.bytes);
        // スプライトを読み込む
        let start_idx = wad.get_lump_index("S_START")?;
        let end_idx = wad.get_lump_index("S_END")?;
        let mut sprites = HashMap::new();
        for lump in &wad.lumps[start_idx + 1..end_idx] {
            sprites.insert(lump.name.clone(), Patch::new_from_bytes(&lump.bytes)?);
        }
        // パッチ名を読み込む
        let pnames = wad.get_lump("PNAMES")?;
        let num_patches = read_i32(&pnames.bytes, 0)? as usize;
        let mut patch_names = Vec::new();
        for i in 0..num_patches {
            let name = read_string(&pnames.bytes, 4 + i * 8, 8)?;
            patch_names.push(name.to_uppercase());
        }
        // テクスチャ用のパッチを読み込む
        let mut texture_patches = HashMap::new();
        for name in &patch_names {
            if let Ok(lump) = wad.get_lump(name) {
                texture_patches.insert(name.clone(), Patch::new_from_bytes(&lump.bytes)?);
            }
        }
        // テクスチャを読み込む
        let mut textures = create_textures(wad, "TEXTURE1", &patch_names, &texture_patches)?;
        if wad.get_lump("TEXTURE2").is_ok() {
            let textures2 = create_textures(wad, "TEXTURE2", &patch_names, &texture_patches)?;
            textures.extend(textures2);
        }
        // Flatを読み込む
        let start_idx = wad.get_lump_index("F_START")?;
        let end_idx = wad.get_lump_index("F_END")?;
        let mut flats = HashMap::new();
        for lump in &wad.lumps[start_idx + 1..end_idx] {
            if lump.bytes.len() != FLAT_SIZE * FLAT_SIZE {
                continue;
            }
            let pallets = lump
                .bytes
                .iter()
                .map(|&idx| idx as usize)
                .collect::<Vec<usize>>();
            flats.insert(lump.name.clone(), pallets);
        }
        Ok(Self {
            palettes,
            sprites,
            patch_names,
            texture_patches,
            textures,
            flats,
        })
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
    pub width: usize,
    pub height: usize,
    pub left_offset: i16,
    pub top_offset: i16,
    pub palettes: Vec<Option<usize>>,
}

impl Patch {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self> {
        let width = read_u16(bytes, 0)? as usize;
        let height = read_u16(bytes, 2)? as usize;
        let mut palettes = vec![None; width * height];
        for x in 0..width {
            // PatchColumnを読み込む
            let offset = read_u32(bytes, 8 + x * 4)? as usize;
            let patch_column = PatchColumn::new_from_bytes(&bytes[offset..])?;
            for (start_y, data) in &patch_column.0 {
                for (offset_y, idx) in data.iter().enumerate() {
                    let y = *start_y as usize + offset_y;
                    palettes[y * width + x] = Some(*idx as usize);
                }
            }
        }
        Ok(Self {
            width,
            height,
            left_offset: read_i16(bytes, 4)?,
            top_offset: read_i16(bytes, 6)?,
            palettes,
        })
    }
}

// (y座標のオフセット, ピクセルデータ)のタプルリスト
struct PatchColumn(pub Vec<(u8, Vec<u8>)>);

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

fn create_textures(
    wad: &Wad,
    name: &str,
    patch_names: &[String],
    pathches: &HashMap<String, Patch>,
) -> Result<HashMap<String, Texture>> {
    let texture = wad.get_lump(name)?;
    let num_textures = read_i32(&texture.bytes, 0)? as usize;
    let offsets = (0..num_textures)
        .map(|i| read_i32(&texture.bytes, 4 + i * 4))
        .collect::<Result<Vec<i32>>>()?;
    let mut textures = HashMap::new();
    for offset in offsets {
        let texture_offset = offset as usize;
        let name = read_string(&texture.bytes, texture_offset, 8)?;
        let width = read_u16(&texture.bytes, texture_offset + 12)? as usize;
        let height = read_u16(&texture.bytes, texture_offset + 14)? as usize;
        let num_patches = read_i16(&texture.bytes, texture_offset + 20)? as usize;
        let mut palettes = vec![None; width * height];
        for i in 0..num_patches {
            let patch_offset = texture_offset + 22 + i * 10;
            let texture_patch = TexturePatch::new_from_bytes(&texture.bytes[patch_offset..])?;
            let patch = &pathches[&patch_names[texture_patch.idx]];
            for x in 0..patch.width {
                for y in 0..patch.height {
                    let tex_x = x as i16 + texture_patch.offset_x;
                    let tex_y = y as i16 + texture_patch.offset_y;
                    if tex_x < 0 || tex_x >= width as i16 || tex_y < 0 || tex_y >= height as i16 {
                        continue;
                    }
                    if let Some(palette_idx) = patch.palettes[y * patch.width + x] {
                        let tex_idx = tex_y as usize * width + tex_x as usize;
                        palettes[tex_idx] = Some(palette_idx);
                    }
                }
            }
        }
        textures.insert(name, Texture::new(width, height, palettes));
    }
    Ok(textures)
}

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub palettes: Vec<Option<usize>>,
}

impl Texture {
    pub fn new(width: usize, height: usize, palettes: Vec<Option<usize>>) -> Self {
        Self {
            width,
            height,
            palettes,
        }
    }
}

struct TexturePatch {
    pub offset_x: i16,
    pub offset_y: i16,
    pub idx: usize,
}

impl TexturePatch {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self> {
        let offset_x = read_i16(bytes, 0)?;
        let offset_y = read_i16(bytes, 2)?;
        let idx = read_i16(bytes, 4)? as usize;
        Ok(Self {
            offset_x,
            offset_y,
            idx,
        })
    }
}
