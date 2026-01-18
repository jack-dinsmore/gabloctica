use cgmath::Vector3;
use rustc_hash::FxHashMap;

use crate::{game::planet::terrain::Terrain, graphics::{CHUNK_SIZE, Graphics}};

use super::Chunk;

pub struct ShipLoader {

}
impl ShipLoader {
    pub(super) fn load_all(&self, graphics: &crate::graphics::Graphics) -> FxHashMap<(i32, i32, i32), Chunk> {
        let chunk_coord = (0, 0, -1);
        let pos = Vector3::new(
            chunk_coord.0 as f32 * CHUNK_SIZE as f32,
            chunk_coord.1 as f32 * CHUNK_SIZE as f32,
            chunk_coord.2 as f32 * CHUNK_SIZE as f32,
        );
        let mut chunk = Chunk::empty(graphics, pos);
        chunk.grid.demo();
        let mut out = FxHashMap::default();
        out.insert(chunk_coord, chunk);
        out
    }
    
    pub(super) fn unload_all(&self) {
        // TODO 
    }
}

pub struct PlanetLoader {
    terrain: Terrain,
    halfwidth: f32,
}
impl PlanetLoader {
    pub fn new(halfwidth: u32, terrain: &Terrain) -> Self {
        Self {
            terrain: terrain.clone(),
            halfwidth: halfwidth as f32,
        }
    }

    pub(super) fn load_chunk(&self, graphics: &Graphics, chunk_coord: (i32, i32, i32)) -> Option<Chunk> {
        // OPTIMIZE clean up given by wind analysis
        let chunk_coord_f = Vector3::new(chunk_coord.0 as f32, chunk_coord.1 as f32, chunk_coord.2 as f32);
        let mut intersecting_faces = Vec::new();
        let mut is_outside = false;
        for face_index in 0..6 {
            let chunk_alt = match face_index {
                0 => chunk_coord_f.x - self.halfwidth,
                1 => -chunk_coord_f.x - self.halfwidth,
                2 => chunk_coord_f.y - self.halfwidth,
                3 => -chunk_coord_f.y - self.halfwidth,
                4 => chunk_coord_f.z - self.halfwidth,
                5 => -chunk_coord_f.z - self.halfwidth,
                _ => unreachable!()
            };
            let val = self.terrain.get_altitude(chunk_coord_f, face_index);
            if chunk_alt > val {
                is_outside = true;
            } else if chunk_alt > val - 1. {
                intersecting_faces.push((face_index, chunk_alt))
            }
        }
        if is_outside {
            return None;
        }
        let pos = chunk_coord_f * CHUNK_SIZE as f32;
        let mut chunk = Chunk::empty(graphics, pos.cast().unwrap());
        if intersecting_faces.is_empty() {
            chunk.grid.demo();
        }

        let recip = 1. / CHUNK_SIZE as f32;
        if intersecting_faces.len() == 1 {
            let (face_index, chunk_alt) = intersecting_faces[0];
            for u in 0..CHUNK_SIZE {
                let ux = u as f32 * recip;
                for v in 0..CHUNK_SIZE {
                    let vx = v as f32 * recip;
                    let block_coord = chunk_coord_f + match face_index {
                        0|1 => Vector3::new(0., ux, vx),
                        2|3 => Vector3::new(ux, 0., vx),
                        4|5 => Vector3::new(ux, vx, 0.),
                        _ => unreachable!()
                    };
                    let local_height = ((self.terrain.get_altitude(block_coord, face_index) - chunk_alt) * CHUNK_SIZE as f32) as u32;

                    for t in 0..local_height.min(CHUNK_SIZE) {
                        match face_index {
                            0 => chunk.grid[(t,u,v)] = 1,
                            1 => chunk.grid[(CHUNK_SIZE-1-t,u,v)] = 1,
                            2 => chunk.grid[(u,t,v)] = 1,
                            3 => chunk.grid[(u,CHUNK_SIZE-1-t,v)] = 1,
                            4 => chunk.grid[(u,v,t)] = 1,
                            5 => chunk.grid[(u,v,CHUNK_SIZE-1-t)] = 1,
                            _ => unreachable!()
                        }
                    }
                }
            }
        }
        if intersecting_faces.len() >= 2 {
            for x in 0..CHUNK_SIZE {
                let xx = x as f32 * recip;
                for y in 0..CHUNK_SIZE {
                    let yx = y as f32 * recip;
                    for z in 0..CHUNK_SIZE {
                        let zx = z as f32 * recip;
                        let block_coord = chunk_coord_f + Vector3::new(xx, yx, zx);
                        let mut outside = false;
                        for (face_index, alt) in &intersecting_faces {
                            let block_alt = alt + match face_index {
                                0 => xx,
                                1 => 1. - xx,
                                2 => yx,
                                3 => 1. - yx,
                                4 => zx,
                                5 => 1. - zx,
                                _ => unreachable!()
                            };
                            // let block_alt = block_coord[0].abs().max(block_coord[1].abs().max(block_coord[2].abs())) - self.halfwidth as f64;
                            if block_alt > self.terrain.get_altitude(block_coord, *face_index) {
                                // Outside
                                outside = true;
                                break;
                            }
                        }
                        if !outside {
                            chunk.grid[(x,y,z)] = 1;
                        }
                    }
                }
            }
        }

        Some(chunk)
    }
    
    pub(super) fn unload_chunk(&self, chunk_coord: (i32, i32, i32), chunk: &Chunk) {
        // TODO 
    }
}