use cgmath::Vector3;
use noise::{NoiseFn, Perlin};
use rustc_hash::FxHashMap;

use crate::graphics::CHUNK_SIZE;

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
    pub(super) fn load_chunk(&self, graphics: &crate::graphics::Graphics, chunk_coord: (i32, i32, i32)) -> Option<Chunk> {
        let linf_norm = chunk_coord.0.abs().max(chunk_coord.1.abs().max(chunk_coord.2.abs()));
        let alt = linf_norm as i32 - self.halfwidth as i32;
        let l2_norm = ((chunk_coord.0*chunk_coord.0 + chunk_coord.1*chunk_coord.1 + chunk_coord.2*chunk_coord.2) as f64).sqrt();
        let ratio = 1. / (linf_norm+1) as f64;
        let sphere_coord = [
            chunk_coord.0 as f64 * ratio,
            chunk_coord.1 as f64 * ratio,
            chunk_coord.2 as f64 * ratio,
        ];
        let val = 3.*self.noise.get(sphere_coord);
        
        if alt as f64>= val {return None;}

        let pos = Vector3::new(
            chunk_coord.0 as f32 * CHUNK_SIZE as f32,
            chunk_coord.1 as f32 * CHUNK_SIZE as f32,
            chunk_coord.2 as f32 * CHUNK_SIZE as f32,
        );
        let mut chunk = Chunk::empty(graphics, pos);
        chunk.grid.demo();
        Some(chunk)
    }
    
    pub(super) fn unload_chunk(&self, chunk_coord: (i32, i32, i32), chunk: &Chunk) {
        // TODO 
    }
}