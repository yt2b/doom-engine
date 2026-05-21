use crate::core::bsp::get_subsector_height;
use crate::core::map::Map;
use crate::core::player::Player;
use crate::core::wad::Wad;
use anyhow::Result;

pub struct Doom {
    pub wad: Wad,
    pub map: Map,
    pub player: Player,
}

impl Doom {
    pub fn new(wad: Wad) -> Result<Self> {
        let map = wad.read_map("E1M1")?;
        let thing = &map.things[0];
        let player = Player::new(thing.x as f32, thing.y as f32, thing.angle as f32);
        Ok(Self { wad, map, player })
    }

    pub fn update(&mut self) {
        let height = get_subsector_height(&self.map, &self.player);
        self.player.set_height(height);
    }
}
