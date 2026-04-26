use crate::platform::game::Game;
use anyhow::Result;
use core::doom::Doom;
use std::fs::File;

mod core;
mod platform;

fn main() -> Result<()> {
    let wad = wad_reader::Wad::new(File::open("./DOOM1.WAD")?)?;
    let doom = Doom::new(wad)?;
    Game::start(doom)
}
