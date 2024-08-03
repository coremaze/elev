use clap::Parser;
use elev::{ElevDump, ElevMap};
use image::{ImageBuffer, Rgb, RgbImage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn load_textures() -> HashMap<u32, Rgb<u8>> {
    let mut textures = HashMap::new();
    for i in 0..=1023 {
        // Assuming texture IDs go from 0 to 1023
        let path = format!("data/terrain{}.jpg", i);
        if let Ok(img) = image::open(&path) {
            let avg_color = average_color(&img.to_rgb8());
            textures.insert(i, avg_color);
        }
    }
    textures
}

fn average_color(img: &RgbImage) -> Rgb<u8> {
    let (r, g, b) = img.pixels().fold((0u64, 0u64, 0u64), |acc, pixel| {
        (
            acc.0 + pixel[0] as u64,
            acc.1 + pixel[1] as u64,
            acc.2 + pixel[2] as u64,
        )
    });
    let pixel_count = img.width() * img.height();
    Rgb([
        (r / pixel_count as u64) as u8,
        (g / pixel_count as u64) as u8,
        (b / pixel_count as u64) as u8,
    ])
}

fn apply_depth(color: Rgb<u8>, height: i32, water_level: i32) -> Rgb<u8> {
    // const WATER_LEVEL: i32 = 1900;
    // const WATER_LEVEL: i32 = 1000;
    const MAX_HEIGHT: f32 = 10_000.0;
    const CONTRAST_CENTER: f32 = 2000.0;
    const CONTRAST_WIDTH: f32 = 1000.0;

    if height <= water_level {
        // Underwater: use a brighter blue gradient
        let depth_factor = (height as f32 / water_level as f32).sqrt();
        let deep_water = Rgb([30, 30, 180]);
        let shallow_water = Rgb([150, 150, 255]);

        return Rgb([
            ((shallow_water[0] as f32 * depth_factor + deep_water[0] as f32 * (1.0 - depth_factor))
                as u8)
                .max(30),
            ((shallow_water[1] as f32 * depth_factor + deep_water[1] as f32 * (1.0 - depth_factor))
                as u8)
                .max(30),
            ((shallow_water[2] as f32 * depth_factor + deep_water[2] as f32 * (1.0 - depth_factor))
                as u8)
                .max(180),
        ]);
    }

    // Custom depth factor calculation to emphasize heights around 2000
    let base_factor = (height as f32 - water_level as f32) / (MAX_HEIGHT - water_level as f32);
    let contrast_factor =
        1.0 / (1.0 + (-4.0 * (height as f32 - CONTRAST_CENTER) / CONTRAST_WIDTH).exp());
    let depth_factor = (base_factor * 0.5 + contrast_factor * 0.5).min(1.0);

    // Blend between 80% brightness and full color based on adjusted depth factor
    let r = (color[0] as f32 * 0.6 + color[0] as f32 * 0.4 * depth_factor) as u8;
    let g = (color[1] as f32 * 0.6 + color[1] as f32 * 0.4 * depth_factor) as u8;
    let b = (color[2] as f32 * 0.6 + color[2] as f32 * 0.4 * depth_factor) as u8;

    Rgb([r, g, b])
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The AW elevdump from which to make an image
    elevdump: PathBuf,

    /// Directory containing textures in the form of "terrain#.jpg"
    texture_dir: PathBuf,

    /// File to produce
    output: PathBuf,

    /// Water level
    #[arg(long)]
    water_level: Option<i32>,
}

fn main() {
    let args = Args::parse();

    let elevdump = match ElevDump::from_file(args.elevdump) {
        Ok(e) => e,
        Err(why) => {
            eprintln!("Failed importing elevdump: {why:#?}");
            return;
        }
    };

    let elev_map = ElevMap::from(&elevdump);
    let textures = load_textures();

    let (min_x, min_z, max_x, max_z) = elev_map.get_bounds();
    let width = (max_x - min_x + 1) * 128;
    let height = (max_z - min_z + 1) * 128;

    let mut img = ImageBuffer::new(width as u32, height as u32);

    for (page_coords, _page) in elev_map.iter_pages() {
        let page_x = page_coords.0;
        let page_z = page_coords.1;

        for x in 0..128u8 {
            for z in 0..128u8 {
                if let Some(cell) = elev_map.get_cell(page_x, page_z, x, z) {
                    let pixel_x = (width - 1) as u32 - ((page_x - min_x) * 128 + x as i32) as u32;
                    let pixel_z = (height - 1) as u32 - ((page_z - min_z) * 128 + z as i32) as u32;

                    let tid = cell.texture_id & 1023;

                    let base_color = textures.get(&tid).unwrap_or(&Rgb([0, 0, 0]));

                    // if !textures.contains_key(&tid) {
                    //     println!("Does not contain {tid}");
                    // }

                    let color =
                        apply_depth(*base_color, cell.height, args.water_level.unwrap_or(0));

                    img.put_pixel(pixel_x, pixel_z, color);
                }
            }
        }
    }

    match img.save(Path::new(&args.output)) {
        Ok(()) => println!("Terrain map saved to {:?}", &args.output),
        Err(why) => eprintln!("Failed to save terrain map: {why:?}"),
    }
}
