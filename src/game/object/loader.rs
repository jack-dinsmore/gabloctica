use cgmath::Vector3;
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

}
impl PlanetLoader {
    pub(super) fn load_chunk(&self, graphics: &crate::graphics::Graphics, chunk_coord: (i32, i32, i32)) -> Option<Chunk> {
        if chunk_coord.2 >= 0 {return None;}
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