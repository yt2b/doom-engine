use crate::platform::game::Game;
use anyhow::{Context, Result};
use clap::Parser;
use core::doom::Doom;
use std::fs::File;

mod core;
mod platform;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "./DOOM1.WAD")]
    wad_path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = File::open(&args.wad_path)
        .with_context(|| format!("指定したWADファイルが見つかりません。 {}", args.wad_path))?;
    let wad = wad_reader::Wad::new(file)?;
    let doom = Doom::new(wad)?;
    Game::start(doom)
}
