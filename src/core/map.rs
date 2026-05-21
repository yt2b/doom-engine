use crate::core::wad::Lump;
use crate::core::wad::{read_i16, read_string};
use anyhow::Result;

#[derive(Debug)]
pub struct Map {
    pub name: String,
    pub things: Vec<Thing>,
    pub linedefs: Vec<Linedef>,
    pub sidedefs: Vec<Sidedef>,
    pub vertexes: Vec<Position>,
    pub segs: Vec<Seg>,
    pub subsectors: Vec<SubSector>,
    pub nodes: Vec<Node>,
    pub sectors: Vec<Sector>,
}

impl Map {
    pub fn new_from_lumps(lumps: &[Lump]) -> Result<Self> {
        let mut map = Self {
            name: lumps[0].name.clone(),
            things: Thing::new_from_bytes(&lumps[1].bytes)?,
            linedefs: Linedef::new_from_bytes(&lumps[2].bytes)?,
            sidedefs: Sidedef::new_from_bytes(&lumps[3].bytes)?,
            vertexes: Position::new_from_bytes(&lumps[4].bytes)?,
            segs: Seg::new_from_bytes(&lumps[5].bytes)?,
            subsectors: SubSector::new_from_bytes(&lumps[6].bytes)?,
            nodes: Node::new_from_bytes(&lumps[7].bytes)?,
            sectors: Sector::new_from_bytes(&lumps[8].bytes)?,
        };
        // Segの表裏の情報を設定する
        for seg in &mut map.segs {
            let linedef = &map.linedefs[seg.line as usize];
            let front_sector = map.sidedefs[linedef.front as usize].sector;
            let exists_back = linedef.flags & 0x0004 != 0;
            let back_sidedef = if exists_back { linedef.back } else { -1 };
            let back_sector = if exists_back {
                map.sidedefs[linedef.back as usize].sector
            } else {
                -1
            };
            if seg.dir == 0 {
                seg.front_sidedef = linedef.front;
                seg.back_sidedef = back_sidedef;
                seg.front_sector = front_sector;
                seg.back_sector = back_sector;
            } else {
                seg.front_sidedef = back_sidedef;
                seg.back_sidedef = linedef.front;
                seg.front_sector = back_sector;
                seg.back_sector = front_sector;
            }
        }
        Ok(map)
    }
}

#[derive(Debug)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle: i16,
    pub kind: i16,
    pub flags: i16,
}

impl Thing {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(size_of::<Thing>())
            .map(|data| {
                Ok(Self {
                    x: read_i16(data, 0)?,
                    y: read_i16(data, 2)?,
                    angle: read_i16(data, 4)?,
                    kind: read_i16(data, 6)?,
                    flags: read_i16(data, 8)?,
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Linedef {
    pub start: i16,
    pub end: i16,
    pub flags: i16,
    pub special: i16,
    pub sector: i16,
    pub front: i16,
    pub back: i16,
}

impl Linedef {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(size_of::<Linedef>())
            .map(|data| {
                Ok(Self {
                    start: read_i16(data, 0)?,
                    end: read_i16(data, 2)?,
                    flags: read_i16(data, 4)?,
                    special: read_i16(data, 6)?,
                    sector: read_i16(data, 8)?,
                    front: read_i16(data, 10)?,
                    back: read_i16(data, 12)?,
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Sidedef {
    pub offset_x: i16,
    pub offset_y: i16,
    pub upper_texture_name: String,
    pub lower_texture_name: String,
    pub middle_texture_name: String,
    pub sector: i16,
}

impl Sidedef {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(30)
            .map(|data| {
                Ok(Self {
                    offset_x: read_i16(data, 0)?,
                    offset_y: read_i16(data, 2)?,
                    upper_texture_name: read_string(data, 4, 8)?,
                    lower_texture_name: read_string(data, 12, 8)?,
                    middle_texture_name: read_string(data, 20, 8)?,
                    sector: read_i16(data, 28)?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(size_of::<Position>())
            .map(|data| Ok(Self::new(read_i16(data, 0)?, read_i16(data, 2)?)))
            .collect()
    }

    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_texture_name: String,
    pub ceiling_texture_name: String,
    pub light_level: i16,
    pub special_kind: i16,
    pub tag: i16,
}

impl Sector {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(26)
            .map(|data| {
                Ok(Self {
                    floor_height: read_i16(data, 0)?,
                    ceiling_height: read_i16(data, 2)?,
                    floor_texture_name: read_string(data, 4, 8)?,
                    ceiling_texture_name: read_string(data, 12, 8)?,
                    light_level: read_i16(data, 20)?,
                    special_kind: read_i16(data, 22)?,
                    tag: read_i16(data, 24)?,
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct SubSector {
    pub seg_count: i16,
    pub seg_idx: i16,
}

impl SubSector {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(size_of::<SubSector>())
            .map(|data| {
                Ok(Self {
                    seg_count: read_i16(data, 0)?,
                    seg_idx: read_i16(data, 2)?,
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Seg {
    pub start: i16,
    pub end: i16,
    pub angle: f32,
    pub line: i16,
    pub dir: i16,
    pub offset_dist: i16,
    pub front_sidedef: i16,
    pub back_sidedef: i16,
    pub front_sector: i16,
    pub back_sector: i16,
}

impl Seg {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(12)
            .map(|data| {
                let angle = (read_i16(data, 4)? as f32) * 360.0 / 65536.0;
                Ok(Self {
                    start: read_i16(data, 0)?,
                    end: read_i16(data, 2)?,
                    angle: if angle >= 0.0 { angle } else { angle + 360.0 },
                    line: read_i16(data, 6)?,
                    dir: read_i16(data, 8)?,
                    offset_dist: read_i16(data, 10)?,
                    front_sidedef: -1,
                    back_sidedef: -1,
                    front_sector: -1,
                    back_sector: -1,
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Node {
    pub start_x: i16,
    pub start_y: i16,
    pub diff_x: i16,
    pub diff_y: i16,
    pub front_bounding: Rect,
    pub back_bounding: Rect,
    pub front_child: i16,
    pub back_child: i16,
}

impl Node {
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        bytes
            .chunks(size_of::<Node>())
            .map(|data| {
                Ok(Self {
                    start_x: read_i16(data, 0)?,
                    start_y: read_i16(data, 2)?,
                    diff_x: read_i16(data, 4)?,
                    diff_y: read_i16(data, 6)?,
                    front_bounding: Rect {
                        top: read_i16(data, 8)?,
                        bottom: read_i16(data, 10)?,
                        left: read_i16(data, 12)?,
                        right: read_i16(data, 14)?,
                    },
                    back_bounding: Rect {
                        top: read_i16(data, 16)?,
                        bottom: read_i16(data, 18)?,
                        left: read_i16(data, 20)?,
                        right: read_i16(data, 22)?,
                    },
                    front_child: read_i16(data, 24)?,
                    back_child: read_i16(data, 26)?,
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Rect {
    pub top: i16,
    pub bottom: i16,
    pub left: i16,
    pub right: i16,
}
