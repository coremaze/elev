mod terrain_mesh;
mod viewer;

use viewer::ViewerSettings;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run() {
    viewer::run(ViewerSettings {
        elevdump: "mion.txt".into(),
        texture_dir: PathBuf::default(),
        water_level: Some(1850),
    });
}

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The AW elevdump to view
    elevdump: PathBuf,

    /// Directory containing textures in the form of "terrain#.jpg"
    texture_dir: PathBuf,

    /// Water level
    #[arg(long)]
    water_level: Option<i32>,
}

fn main() {
    let args = Args::parse();
    viewer::run(ViewerSettings {
        elevdump: args.elevdump,
        texture_dir: std::env::current_dir()
            .unwrap_or_default()
            .join(args.texture_dir),
        water_level: args.water_level,
    });
}
