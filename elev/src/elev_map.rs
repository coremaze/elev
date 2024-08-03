use std::collections::HashMap;

use super::{ElevDump, ElevEntry};

#[derive(Debug, Clone, Copy)]
pub struct ElevCell {
    pub texture_id: u32,
    pub rotation: Rotation,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Rotation {
    #[default]
    R0,
    R1,
    R2,
    R3,
}

#[derive(Debug)]
pub struct ElevPage {
    cells: [[ElevCell; 128]; 128],
}

impl ElevPage {
    fn new() -> Self {
        ElevPage {
            cells: [[ElevCell {
                texture_id: 0,
                height: 0,
                rotation: Default::default(),
            }; 128]; 128],
        }
    }

    pub fn get_cell(&self, x: u8, z: u8) -> Option<&ElevCell> {
        let Some(z_cells) = self.cells.get(usize::from(z)) else {
            return None;
        };

        let Some(zx_cell) = z_cells.get(usize::from(x)) else {
            return None;
        };

        Some(zx_cell)
    }

    fn set_cell(&mut self, x: u8, z: u8, cell: ElevCell) {
        let Some(z_cells) = self.cells.get_mut(usize::from(z)) else {
            return;
        };

        let Some(zx_cell) = z_cells.get_mut(usize::from(x)) else {
            return;
        };

        *zx_cell = cell;
    }
}

#[derive(Debug)]
pub struct ElevMap {
    pages: HashMap<(i32, i32), ElevPage>,
}

impl ElevMap {
    pub fn get_cell(&self, page_x: i32, page_z: i32, x: u8, z: u8) -> Option<&ElevCell> {
        self.pages
            .get(&(page_x, page_z))
            .and_then(|page| page.get_cell(x, z))
    }

    fn set_cell(&mut self, page_x: i32, page_z: i32, x: u8, z: u8, cell: ElevCell) {
        self.pages
            .entry((page_x, page_z))
            .or_insert_with(ElevPage::new)
            .set_cell(x, z, cell);
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn iter_pages(&self) -> impl Iterator<Item = (&(i32, i32), &ElevPage)> {
        self.pages.iter()
    }

    pub fn get_bounds(&self) -> (i32, i32, i32, i32) {
        let mut min_x = i32::MAX;
        let mut min_z = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_z = i32::MIN;

        for &(page_x, page_z) in self.pages.keys() {
            min_x = min_x.min(page_x);
            min_z = min_z.min(page_z);
            max_x = max_x.max(page_x);
            max_z = max_z.max(page_z);
        }

        (min_x, min_z, max_x, max_z)
    }
}

impl From<&ElevDump> for ElevMap {
    fn from(dump: &ElevDump) -> Self {
        let mut map = ElevMap {
            pages: HashMap::new(),
        };

        for entry in &dump.entries {
            map.apply_entry(entry);
        }

        map
    }
}

impl ElevMap {
    fn apply_entry(&mut self, entry: &ElevEntry) {
        let diameter = (entry.node_radius as u16) * 2;
        // println!("{entry:#?}");

        for dx in 0..diameter {
            for dz in 0..diameter {
                let x = entry.node_x as u16 + dx;
                let z = entry.node_z as u16 + dz;

                // println!("{dx} {dz}");

                if x < 128 && z < 128 {
                    let index = (dz * diameter + dx) as usize;

                    let mut texture_id = *entry
                        .texture_ids
                        .get(index)
                        .unwrap_or(entry.texture_ids.get(0).unwrap_or(&0));

                    let rotation = match texture_id & 0b1100_0000_0000_0000 {
                        0x8000 => Rotation::R0,
                        0x4000 => Rotation::R1,
                        0x0000 => Rotation::R2,
                        0xC000 => Rotation::R3,
                        _ => panic!("impossible rotation"),
                    };
                    texture_id = texture_id & (!0b1100_0000_0000_0000);

                    let &height = entry
                        .heights
                        .get(index)
                        .unwrap_or(entry.heights.get(0).unwrap_or(&0));

                    self.set_cell(
                        entry.page_x,
                        entry.page_z,
                        x as u8,
                        z as u8,
                        ElevCell {
                            texture_id,
                            height,
                            rotation,
                        },
                    );
                }
            }
        }
    }
}
