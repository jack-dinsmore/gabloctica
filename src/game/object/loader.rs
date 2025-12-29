use cgmath::{Vector3, Zero};
use faer::Mat;

use crate::{game::object::chunk, graphics::{CHUNK_SIZE, CubeGrid}};

use super::Chunk;

pub struct ShipLoader {

}
impl ShipLoader {
    pub(super) fn load_all(&self, graphics: &crate::graphics::Graphics) -> (Vec<(i32, i32, i32)>, Vec<Chunk>) {
        let pos = (0, 0, 0);
        let mut grid = CubeGrid::new(graphics);
        grid.demo(graphics);
        let chunk = Chunk {
            grid,
            mass_m0: 0.,
            mass_m1: Vector3::zero(),
            mass_m2: Mat::zeros(3, 3),
        };
        (vec![pos], vec![chunk])
    }
    
    pub(super) fn unload_all(&self) {
        // TODO 
    }
}

pub struct PlanetLoader {

}
impl PlanetLoader {
    pub(super) fn load_chunk(&self, graphics: &crate::graphics::Graphics, chunk_coord: (i32, i32, i32)) -> Option<Chunk> {
        if chunk_coord.2 < 0 {return None;}
        let pos = Vector3::new(
            chunk_coord.0 as f32 * CHUNK_SIZE as f32,
            chunk_coord.1 as f32 * CHUNK_SIZE as f32,
            chunk_coord.2 as f32 * CHUNK_SIZE as f32,
        );
        let mut chunk = Chunk::empty(graphics, pos);
        chunk.grid.demo(graphics);
        Some(chunk)
    }
    pub(super) fn unload_chunk(&self, chunk_coord: (i32, i32, i32), chunk: &Chunk) {
        // TODO 
    }
}