use std::path::PathBuf;

use bevy::prelude::*;
use bevy_flycam::prelude::*;
use elev::{ElevDump, ElevMap};
use terrain_mesh::create_terrain_meshes;

use crate::terrain_mesh;

#[derive(Resource)]
pub struct ViewerSettings {
    pub elevdump: PathBuf,
    pub texture_dir: PathBuf,
    pub water_level: Option<i32>,
}

pub fn run(settings: ViewerSettings) {
    App::new()
        .insert_resource(settings)
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .insert_resource(ClearColor(Color::srgb(0.6, 0.7, 0.95))) // Sky blue color
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    settings: Res<ViewerSettings>,
) {
    // println!("Setup");
    let elev_map = ElevMap::from(&ElevDump::from_file(&settings.elevdump).unwrap());
    // println!("elev_map made");

    // Create the terrain mesh
    let terrain_meshes = create_terrain_meshes(&elev_map);
    // println!("terrain_meshes made");

    for (texture_id, mesh) in terrain_meshes {
        let texture_path = settings
            .texture_dir
            .join(format!("terrain{texture_id}.jpg"));
        let texture_handle = asset_server.load(texture_path);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            metallic: 0.0,
            perceptual_roughness: 1.0, // Lower values make it appear more reflective
            reflectance: 0.1,
            ..default()
        });
        // Spawn the terrain mesh
        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: material.clone(),
            // material: materials.add(Color::srgb_u8(255, 255, 255)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });

        let cube_x = texture_id % 16;
        let cube_z = texture_id / 16;

        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
            material,
            transform: Transform::from_xyz(cube_x as f32, -10.0, cube_z as f32),
            ..default()
        });
    }
    // println!("terrain_meshes spawned");

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: false,
            color: Color::WHITE,
            ..default()
        },
        // Set the transform to point the light in the desired direction
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // water plane
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.5, 1.0, 0.9),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.0,
        perceptual_roughness: 0.1,
        ..default()
    });
    if let Some(water_level) = settings.water_level {
        commands.spawn(PbrBundle {
            mesh: meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(128000.0, 12800.0)
                    .subdivisions(10),
            ),
            material: water_material,
            transform: Transform::from_xyz(0.0, water_level as f32 / 1000.0, 0.0),
            ..default()
        });
    }
}
