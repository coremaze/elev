use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use elev::{ElevCell, ElevMap, Rotation};

pub fn create_terrain_meshes(elev_map: &ElevMap) -> HashMap<u32, Mesh> {
    let mut mesh_map: HashMap<u32, MeshBuilderData> = HashMap::new();

    let (min_page_x, min_page_z, max_page_x, max_page_z) = elev_map.get_bounds();

    // Helper function to get cell, handling page boundaries
    let get_cell = |page_x: i32, page_z: i32, x: i32, z: i32| -> Option<&ElevCell> {
        let (target_page_x, target_x) = if x == 128 {
            (page_x + 1, 0)
        } else {
            (page_x, x)
        };
        let (target_page_z, target_z) = if z == 128 {
            (page_z + 1, 0)
        } else {
            (page_z, z)
        };

        elev_map.get_cell(target_page_x, target_page_z, target_x as u8, target_z as u8)
    };

    // println!("Started adding quads");
    for page_z in min_page_z..=max_page_z {
        for page_x in min_page_x..=max_page_x {
            for z in 0..128 {
                for x in 0..128 {
                    if let Some(cell) = get_cell(page_x, page_z, x, z) {
                        let mesh_data = mesh_map
                            .entry(cell.texture_id)
                            .or_insert_with(MeshBuilderData::new);

                        let world_x = x as f32 + (page_x * 128) as f32;
                        let world_z = z as f32 + (page_z * 128) as f32;

                        let cell0 = cell;
                        let Some(cell1) = get_cell(page_x, page_z, x + 1, z) else {
                            continue;
                        };
                        let Some(cell2) = get_cell(page_x, page_z, x, z + 1) else {
                            continue;
                        };
                        let Some(cell3) = get_cell(page_x, page_z, x + 1, z + 1) else {
                            continue;
                        };
                        let v0 = [world_x, cell0.height as f32 / 1000.0, world_z];
                        let v1 = [world_x + 1.0, cell1.height as f32 / 1000.0, world_z];
                        let v2 = [world_x, cell2.height as f32 / 1000.0, world_z + 1.0];
                        let v3 = [world_x + 1.0, cell3.height as f32 / 1000.0, world_z + 1.0];

                        let rotated_uvs = rotate_uvs(cell.rotation);

                        mesh_data.add_quad(v0, v1, v2, v3, rotated_uvs);
                    }
                }
            }
        }
    }

    // println!("Done adding quads");

    mesh_map
        .into_iter()
        .map(|(texture_id, data)| {
            let mesh = data.build_mesh();
            (texture_id, mesh)
        })
        .collect()
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct VertexKey {
    position: [i32; 3],
    uv: [u16; 2],
}

impl VertexKey {
    fn new(position: [f32; 3], uv: [f32; 2]) -> Self {
        VertexKey {
            position: [
                (position[0] * 1000.0) as i32,
                (position[1] * 1000.0) as i32,
                (position[2] * 1000.0) as i32,
            ],
            uv: [(uv[0] * 65535.0) as u16, (uv[1] * 65535.0) as u16],
        }
    }
}

struct MeshBuilderData {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    vertex_map: HashMap<VertexKey, u32>,
}

impl MeshBuilderData {
    fn new() -> Self {
        MeshBuilderData {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            vertex_map: HashMap::new(),
        }
    }

    // fn add_quad(
    //     &mut self,
    //     v0: [f32; 3],
    //     v1: [f32; 3],
    //     v2: [f32; 3],
    //     v3: [f32; 3],
    //     uvs: [[f32; 2]; 4],
    // ) {
    //     let i0 = self.add_vertex(v0, [0.0, 1.0, 0.0], uvs[0]);
    //     let i1 = self.add_vertex(v1, [0.0, 1.0, 0.0], uvs[1]);
    //     let i2 = self.add_vertex(v2, [0.0, 1.0, 0.0], uvs[2]);
    //     let i3 = self.add_vertex(v3, [0.0, 1.0, 0.0], uvs[3]);

    //     self.indices.extend_from_slice(&[i0, i3, i1, i0, i2, i3]);
    // }

    fn add_quad(
        &mut self,
        v0: [f32; 3],
        v1: [f32; 3],
        v2: [f32; 3],
        v3: [f32; 3],
        uvs: [[f32; 2]; 4],
    ) {
        fn calculate_quad_normal(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
            // Calculate two diagonal vectors
            let diagonal1 = v0 - v3;
            let diagonal2 = v2 - v1;

            // Calculate the cross product
            let normal = diagonal1.cross(diagonal2);

            // Normalize the result
            normal.normalize()
        }

        let v0_v = Vec3::from_array(v0);
        let v1_v = Vec3::from_array(v1);
        let v2_v = Vec3::from_array(v2);
        let v3_v = Vec3::from_array(v3);

        let normal = calculate_quad_normal(v0_v, v1_v, v2_v, v3_v).to_array();

        // Calculate the lengths of the two diagonals
        let diagonal0_length = (v0_v - v3_v).length_squared();
        let diagonal1_length = (v1_v - v2_v).length_squared();

        let i0_1 = self.add_vertex(v0, normal, uvs[0]);
        let i1_1 = self.add_vertex(v1, normal, uvs[1]);
        let i2_1 = self.add_vertex(v2, normal, uvs[2]);
        let i3_1 = self.add_vertex(v3, normal, uvs[3]);

        if diagonal0_length >= diagonal1_length {
            // Split along v0-v3

            self.indices
                .extend_from_slice(&[i0_1, i3_1, i1_1, i0_1, i2_1, i3_1]);
        } else {
            // Split along v1-v2
            self.indices
                .extend_from_slice(&[i1_1, i2_1, i3_1, i1_1, i0_1, i2_1]);
        }
    }

    fn add_vertex(&mut self, position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> u32 {
        let key = VertexKey::new(position, uv);
        if let Some(&index) = self.vertex_map.get(&key) {
            return index;
        }

        let index = self.vertices.len() as u32;
        self.vertices.push(position);
        self.normals.push(normal);
        self.uvs.push(uv);
        self.vertex_map.insert(key, index);
        index
    }

    fn build_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

fn rotate_uvs(rotation: Rotation) -> [[f32; 2]; 4] {
    match rotation {
        Rotation::R0 => [[1.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        Rotation::R1 => [[1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [0.0, 1.0]],
        Rotation::R2 => [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        Rotation::R3 => [[0.0, 1.0], [0.0, 0.0], [1.0, 1.0], [1.0, 0.0]],
    }
}
