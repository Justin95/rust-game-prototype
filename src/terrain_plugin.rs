use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{AddressMode, FilterMode};
use crate::{player, terrain};
use crate::terrain::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Terrain::new())
            .insert_resource(TerrainMaterials {
                grass: None
            })
            .add_system(texture_load_listener)
            .add_startup_system(initialize_mats)
            .add_startup_system(initialize)
            .add_system(manage_terrain);
    }
}

#[derive(Component)]
struct Terrain {
    terrain_data: TerrainData,
    terrain_entities: HashMap<TerrainLocation, Entity>
}

struct TerrainMaterials {
    grass: Option<Handle<StandardMaterial>>
}

impl Terrain {
    pub fn new() -> Terrain {
        return Terrain {
            terrain_data: TerrainData::new(),
            terrain_entities: HashMap::new()
        };
    }
}

fn texture_load_listener(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>
) {
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created {handle} => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    println!("texture modified");
                    //texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST;
                    texture.sampler_descriptor.address_mode_u = AddressMode::Repeat;
                    texture.sampler_descriptor.address_mode_v = AddressMode::Repeat;
                    texture.sampler_descriptor.address_mode_w = AddressMode::Repeat;
                    texture.sampler_descriptor.mag_filter = FilterMode::Linear;
                    texture.sampler_descriptor.min_filter = FilterMode::Linear;
                }
            }
            _ => {},
        }
    }
}

fn initialize_mats(
    asset_server: Res<AssetServer>,
    mut terrain_mats_res: ResMut<TerrainMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let grass_img = asset_server.load("grass_color.png");
    let grass_mat = StandardMaterial {
        base_color_texture: Some(grass_img),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.0,
        reflectance: 0.2,
        perceptual_roughness: 0.95,
        unlit: false,
        ..default()
    };
    terrain_mats_res.grass = Some(materials.add(grass_mat));
}

fn initialize(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            color: Color::rgb(0.9, 0.9, 0.8),
            illuminance: 50000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 1000.0, 0.0),
            rotation: Quat::from_rotation_x((-60.0 as f32).to_radians()),
            ..default()
        },
        ..default()
    });
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.3, 0.3, 0.4),
        brightness: 1.0
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 1.0,
            .. default()
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 300.0, 0.0),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 1.0,
            .. default()
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 300.0, 3.0),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 1.0,
            .. default()
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::GREEN,
            ..default()
        }),
        transform: Transform::from_xyz(-0.5, 300.0, 0.0),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 1.0,
            .. default()
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..default()
        }),
        transform: Transform::from_xyz(3.0, 300.0, 0.0),
        ..default()
    });
}

fn manage_terrain(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut terrain_res: ResMut<Terrain>,
    terrain_mats_res: Res<TerrainMaterials>,
    player_query: Query<&Transform, With<player::Player>>
) {
    //there should only ever be one player but look for one defensively
    let mut player_loc_opt: Option<Vec3> = Option::None;
    for transform in player_query.iter() {
        player_loc_opt = Option::Some(transform.translation);
        break;
    }
    if player_loc_opt.is_none() {
        return; //shouldn't happen, but if it does we cant update terrain
    }
    let grass_material = terrain_mats_res.grass.clone().unwrap();
    let terrain = terrain_res.as_mut();
    let terrain_entities = &mut terrain.terrain_entities;
    let terrain_data = &mut terrain.terrain_data;
    let player_loc = player_loc_opt.unwrap();
    let load_center = TerrainLocation {
        x: (player_loc.x / TERRAIN_PIECE_WORLD_LENGTH as f32).floor() as i32,
        z: (player_loc.z / TERRAIN_PIECE_WORLD_LENGTH as f32).floor() as i32
    };
    if !terrain::require_update(terrain_data, &load_center) {
        return;
    }
    
    terrain::update_terrain(terrain_data, &load_center);
    
    //remove out of load distance terrain
    let mut to_remove: Vec<TerrainLocation> = Vec::new();
    for terrain_loc in terrain_entities.keys() {
        if terrain_loc.x < load_center.x - terrain::LOAD_DIST || terrain_loc.x > load_center.x + terrain::LOAD_DIST
        || terrain_loc.z < load_center.z - terrain::LOAD_DIST || terrain_loc.z > load_center.z + terrain::LOAD_DIST {
            let entity_opt= terrain_entities.get(terrain_loc);
            if entity_opt.is_none() {
                continue;
            }
            to_remove.push(terrain_loc.clone());
            let entity = entity_opt.unwrap().clone();
            commands.entity(entity).despawn();
        }
    }
    for terrain_loc in to_remove {
        terrain_entities.remove(&terrain_loc);
    }
    
    let render_dist = terrain::LOAD_DIST;
    for x in load_center.x - render_dist .. load_center.x + render_dist {
        for z in load_center.z - render_dist .. load_center.z + render_dist {
            let terrain_loc = TerrainLocation {x: x, z: z};
            if terrain_entities.contains_key(&terrain_loc) {
                continue;
            }
            let terrain_piece_opt = terrain_data.get_terrain_piece(&terrain_loc);
            if terrain_piece_opt.is_none() {
                continue;
            }
            let terrain_piece = terrain_piece_opt.unwrap();
            let pbr_bundle = generate_pbr_bundle(terrain_piece, &mut meshes, grass_material.clone());
            let entity = commands
                .spawn_bundle(pbr_bundle)
                .id();
            terrain_entities.insert(terrain_loc, entity);
        }
    }
    
    //http://clynamen.github.io/blog/2021/01/04/terrain_generation_bevy/
    //https://textures.pixel-furnace.com/
    //https://stackoverflow.com/questions/66677098/how-can-i-manually-create-meshes-in-bevy-with-vertices
}

fn generate_pbr_bundle(terrain_piece: TerrainPiece, meshes: &mut ResMut<Assets<Mesh>>, material: Handle<StandardMaterial>) -> PbrBundle {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut positions: Vec<[f32;3]> = Vec::new();
    let mut normals: Vec<[f32;3]> = Vec::new();
    let mut uvs: Vec<[f32;2]> = Vec::new();
    let terrain_scale = TERRAIN_POINT_WORLD_DIST;
    for z in 0 .. TERRAIN_PIECE_POINTS_PER_LENGTH {
        for x in 0 .. TERRAIN_PIECE_POINTS_PER_LENGTH {
            let fx = (x as f64 * terrain_scale) as f32;
            let fz = (z as f64 * terrain_scale) as f32;
            let fy = terrain_piece.terrain_data[(x + z * TERRAIN_PIECE_POINTS_PER_LENGTH) as usize];
            positions.push([fx, fy, fz]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([fx / TERRAIN_PIECE_POINTS_PER_LENGTH as f32, fz / TERRAIN_PIECE_POINTS_PER_LENGTH as f32]);
        }
    }
    let mut indices: Vec<u16> = Vec::new();
    for z in 0 .. TERRAIN_PIECE_POINTS_PER_LENGTH - 1 {
        for x in 0 .. TERRAIN_PIECE_POINTS_PER_LENGTH - 1 {
            indices.push((x + z * TERRAIN_PIECE_POINTS_PER_LENGTH) as u16);
            indices.push((x + 1 + (z + 1) * TERRAIN_PIECE_POINTS_PER_LENGTH) as u16);
            indices.push((x + 1 + z * TERRAIN_PIECE_POINTS_PER_LENGTH) as u16);
            indices.push((x + z * TERRAIN_PIECE_POINTS_PER_LENGTH) as u16);
            indices.push((x + (z + 1) * TERRAIN_PIECE_POINTS_PER_LENGTH) as u16);
            indices.push((x + 1 + (z + 1) * TERRAIN_PIECE_POINTS_PER_LENGTH) as u16);
        }
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U16(indices)));

    let translation: Vec3 = Vec3::new(
        (terrain_piece.position.x as f64 * TERRAIN_PIECE_WORLD_LENGTH) as f32,
        0.0,
        (terrain_piece.position.z as f64 * TERRAIN_PIECE_WORLD_LENGTH) as f32
    );

    return PbrBundle {
        mesh: meshes.add(mesh),
        material: material,
        transform: Transform::from_translation(translation),
        ..default()
    }
}