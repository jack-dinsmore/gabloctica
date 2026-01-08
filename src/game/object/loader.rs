use cgmath::Vector3;
use noise::{NoiseFn, Perlin};
use rustc_hash::FxHashMap;

use crate::graphics::{CHUNK_SIZE, Graphics};

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
    halfwidth: u32,
    noise: Perlin,
}
impl PlanetLoader {
    pub fn new(halfwidth: u32, noise: Perlin) -> Self {
        Self {
            halfwidth,
            noise,
        }
    }

    fn altitude(&self, pos: [f64; 3]) -> f64 {
        3.*self.noise.get([pos[0], pos[1], pos[2]])
    }

    pub(super) fn load_chunk(&self, graphics: &Graphics, chunk_coord: (i32, i32, i32)) -> Option<Chunk> {
        let linf_norm = chunk_coord.0.abs().max(chunk_coord.1.abs().max(chunk_coord.2.abs()));
        let alt = linf_norm as f64 - self.halfwidth as f64;
        let l2_norm = ((chunk_coord.0*chunk_coord.0 + chunk_coord.1*chunk_coord.1 + chunk_coord.2*chunk_coord.2) as f64).sqrt();
        let ratio = 1. / (l2_norm+1.);
        let sphere_coord = [
            chunk_coord.0 as f64 * ratio,
            chunk_coord.1 as f64 * ratio,
            chunk_coord.2 as f64 * ratio,
        ];
        // dbg!(sphere_coord[0]*sphere_coord[0] + sphere_coord[1]*sphere_coord[1] + sphere_coord[2]*sphere_coord[2]);
        let val = self.altitude(sphere_coord);
        
        if alt >= val + 1. {return None;}

        let pos = Vector3::new(
            chunk_coord.0 as f32 * CHUNK_SIZE as f32,
            chunk_coord.1 as f32 * CHUNK_SIZE as f32,
            chunk_coord.2 as f32 * CHUNK_SIZE as f32,
        );
        let mut chunk = Chunk::empty(graphics, pos);
        if alt < val - 1. {
            chunk.grid.demo();
        } else {
            let mut face_index = 0;
            let little_ratio = ratio / (CHUNK_SIZE as f64);
            if linf_norm == chunk_coord.0.abs() { face_index = if chunk_coord.0 > 0 { 0 } else { 1 }}
            if linf_norm == chunk_coord.1.abs() { face_index = if chunk_coord.1 > 0 { 2 } else { 3 }}
            if linf_norm == chunk_coord.2.abs() { face_index = if chunk_coord.2 > 0 { 4 } else { 5 }}

            for u in 0..CHUNK_SIZE {
                let ux = u as f64*little_ratio;
                for v in 0..CHUNK_SIZE {
                    let vx = v as f64*little_ratio;
                    let sphere_coord = match face_index {
                        0|1 => [sphere_coord[0], sphere_coord[1] + ux, sphere_coord[2] + vx],
                        2|3 => [sphere_coord[0] + ux, sphere_coord[1], sphere_coord[2] + vx],
                        4|5 => [sphere_coord[0] + ux, sphere_coord[1] + vx, sphere_coord[2]],
                        _ => unreachable!()
                    };
                    let local_height = ((self.altitude(sphere_coord) - alt) * CHUNK_SIZE as f64) as u32;

                    if val < 0. {continue;}
                    for t in 0..local_height.min(CHUNK_SIZE) {
                        match face_index {
                            0 => chunk.grid[(t,u,v)] = 2,
                            1 => chunk.grid[(CHUNK_SIZE-1-t,u,v)] = 2,
                            2 => chunk.grid[(u,t,v)] = 2,
                            3 => chunk.grid[(u,CHUNK_SIZE-1-t,v)] = 2,
                            4 => chunk.grid[(u,v,t)] = 2,
                            5 => chunk.grid[(u,v,CHUNK_SIZE-1-t)] = 2,
                            _ => unreachable!()
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