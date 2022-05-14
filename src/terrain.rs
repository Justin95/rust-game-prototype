use std::collections::HashMap;
use noise::{NoiseFn, OpenSimplex, Seedable};

const TERRAIN_SEED: u32 = 44;
pub const TERRAIN_PIECE_WORLD_LENGTH: f64 = 50.0;
pub const TERRAIN_PIECE_POINTS_PER_LENGTH: i32 = 50; //min value 2
pub const TERRAIN_POINT_WORLD_DIST: f64 = TERRAIN_PIECE_WORLD_LENGTH / (TERRAIN_PIECE_POINTS_PER_LENGTH - 1) as f64;
const TERRAIN_DATA_SIZE: usize = (TERRAIN_PIECE_POINTS_PER_LENGTH * TERRAIN_PIECE_POINTS_PER_LENGTH) as usize;
pub const LOAD_DIST: i32 = 20;

pub struct TerrainData {
    terrain_pieces: HashMap<TerrainLocation, TerrainPiece>,
    load_center: TerrainLocation,
    fully_loaded: bool
}

impl TerrainData {
    pub fn new() -> TerrainData {
        return TerrainData {
            terrain_pieces: HashMap::new(),
            load_center: TerrainLocation {x: 0, z: 0},
            fully_loaded: false
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct TerrainLocation {
    pub x: i32,
    pub z: i32
}

#[derive(Clone)]
pub struct TerrainPiece {
    pub position: TerrainLocation,
    pub terrain_data: Box<[f32; TERRAIN_DATA_SIZE]>
}

pub fn require_update(terrain_data: &mut TerrainData, center: &TerrainLocation) -> bool {
    if terrain_data.load_center.x != center.x || terrain_data.load_center.z != center.z {
        terrain_data.fully_loaded = false;
    }
    return !terrain_data.fully_loaded;
}

pub fn update_terrain(terrain_data: &mut TerrainData, center: &TerrainLocation) {
    terrain_data.load_center = center.clone();
    let load_loc_opt = get_next_load_loc(terrain_data, &center);
    if load_loc_opt.is_none() {
        terrain_data.fully_loaded = true;
        return;
    }
    let load_loc = load_loc_opt.unwrap();
    let terrain_piece = generate_terrain_piece(load_loc);
    terrain_data.terrain_pieces.insert(load_loc, terrain_piece);
}

fn get_next_load_loc(terrain_data: &TerrainData, center: &TerrainLocation) -> Option<TerrainLocation> {
    for i in 0 .. LOAD_DIST {
        //x+ -z->+z, +z +x->-x, -x +z->-z, -z -x->+x
        //x+ -z->+z
        for z in -i .. i + 1 {
            let loc = TerrainLocation {
                x: center.x + i,
                z: center.z + z
            };
            if !terrain_data.terrain_pieces.contains_key(&loc) {
                return Some(loc);
            }
        }
        //+z -x->+x
        for x in -i - 1 .. i {
            let loc = TerrainLocation {
                x: center.x + x,
                z: center.z + i
            };
            if !terrain_data.terrain_pieces.contains_key(&loc) {
                return Some(loc);
            }
        }
        //-x -z->+z
        for z in -i - 1 .. i {
            let loc = TerrainLocation {
                x: center.x - i,
                z: center.z + z
            };
            if !terrain_data.terrain_pieces.contains_key(&loc) {
                return Some(loc);
            }
        }
        //-z -x->+x
        for x in -i .. i + 1 {
            let loc = TerrainLocation {
                x: center.x + x,
                z: center.z - i
            };
            if !terrain_data.terrain_pieces.contains_key(&loc) {
                return Some(loc);
            }
        }
    }
    return None;
}

impl TerrainData {
    pub fn get_terrain_piece(&self, loc: &TerrainLocation) -> Option<TerrainPiece> {
        return self.terrain_pieces.get(loc).cloned();
    }
}

fn generate_terrain_piece(pos: TerrainLocation) -> TerrainPiece {
    let noise = OpenSimplex::new();
    noise.set_seed(TERRAIN_SEED);
    let mut terrain_data: Box<[f32; TERRAIN_DATA_SIZE]> = Box::new([0.0; TERRAIN_DATA_SIZE]); //dumb mem copy, thanks rust
    for z in 0 .. TERRAIN_PIECE_POINTS_PER_LENGTH {
        for x in 0 .. TERRAIN_PIECE_POINTS_PER_LENGTH {
            let fx = (x) as f64 * TERRAIN_POINT_WORLD_DIST + (TERRAIN_PIECE_WORLD_LENGTH * pos.x as f64);
            let fz = (z) as f64 * TERRAIN_POINT_WORLD_DIST + (TERRAIN_PIECE_WORLD_LENGTH * pos.z as f64);
            terrain_data[(x + z * TERRAIN_PIECE_POINTS_PER_LENGTH) as usize] = noise_at(noise, fx, fz);
        }
    }
    return TerrainPiece {
        position: pos,
        terrain_data: terrain_data
    };
}

fn noise_at(noise: OpenSimplex, x: f64, y: f64) -> f32 {
    let val = noise.get([x / 500.0, y / 500.0]) * 200.0
        + noise.get([x / 200.0, y / 200.0]) * 100.0
        + noise.get([x / 50.0, y / 50.0]) * 30.0
        + noise.get([x / 10.0, y / 10.0]) * 10.0
        + noise.get([x / 2.0, y / 2.0]) * 2.0;
    return val as f32;
}