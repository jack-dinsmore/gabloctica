use cgmath::Vector3;
use rustc_hash::FxHashMap;

use crate::{game::{object::{BlockProperties, chunk::{PhysicalBlock, PhysicalChunk}}, planet::Planet}, graphics::Graphics, util::CHUNK_SIZE};

pub struct ShipLoader {
    pub pos: Vector3<f64>,
    pub vel: Vector3<f64>,
}
impl ShipLoader {
    pub(super) fn load_all(&self, graphics: &Graphics, properties: &BlockProperties) -> FxHashMap<(i32, i32, i32), PhysicalChunk> {
        let chunk_coord = (0, 0, 0);
        let mut chunk = PhysicalChunk::empty(graphics, chunk_coord);

        chunk.grid[(7,7,7)] = PhysicalBlock::new(7, 0, properties);// Metal
        chunk.grid[(6,8,7)] = PhysicalBlock::new(4, 1, properties);// Tank
        chunk.grid[(7,8,7)] = PhysicalBlock::new(4, 1, properties);// Tank
        chunk.grid[(8,8,7)] = PhysicalBlock::new(5, 3, properties);// Engine
        chunk.grid[(6,6,7)] = PhysicalBlock::new(4, 1, properties);// Tank
        chunk.grid[(7,6,7)] = PhysicalBlock::new(4, 1, properties);// Tank
        chunk.grid[(8,6,7)] = PhysicalBlock::new(5, 3, properties);// Engine
        chunk.grid[(7,7,8)] = PhysicalBlock::new(6, 0, properties);// Chair
        // chunk.grid[(8,8,8)] = PhysicalBlock::new(8, 0, properties);// Rock

        let mut out = FxHashMap::default();
        out.insert(chunk_coord, chunk);
        out
    }
    
    pub(super) fn unload_all(&self) {
        // TODO implement unloading ships so that they can be reloaded with changes saved
    }
}

pub struct PlanetLoader {
    planet: Planet,
    halfwidth: f32,
}
impl PlanetLoader {
    pub fn new(halfwidth: u32, planet: &Planet) -> Self {
        Self {
            planet: planet.clone(),//OPTIMIZE
            halfwidth: halfwidth as f32,
        }
    }

    pub(super) fn load_chunk(&self, graphics: &Graphics, properties: &BlockProperties, chunk_coord: (i32, i32, i32)) -> Option<PhysicalChunk> {
        let chunk_pos = Vector3::new(chunk_coord.0 as f32, chunk_coord.1 as f32, chunk_coord.2 as f32) * CHUNK_SIZE as f32;
        if self.planet.get_point_altitude(chunk_pos) > self.planet.get_altitude(chunk_pos) + CHUNK_SIZE as f32 {
            // The chunk is outside the planet. Don't load it.
            return None;
        }

        let mut chunk = PhysicalChunk::empty(graphics, chunk_coord);
        let max = chunk_pos.x.abs().max(chunk_pos.y.abs().max(chunk_pos.z.abs()));
        let face_index = if max == chunk_pos.x.abs() {
            if chunk_pos.x > 0. { 0 } else { 1 }
        } else if max == chunk_pos.y.abs() {
            if chunk_pos.y > 0. { 2 } else { 3 }
        } else {
            if chunk_pos.z > 0. { 4 } else { 5 }
        };

        // Build the cube up from the bottom
        for u in 0..CHUNK_SIZE {
            for v in 0..CHUNK_SIZE {
                for w in 0..CHUNK_SIZE {
                    let pos = chunk_pos + Vector3::new(u as f32, v as f32, w as f32);
                    let depth = (self.planet.get_altitude(pos) - self.planet.get_point_altitude(pos)) as i32;
                    if depth < 0 {continue;}
                    let biome = self.planet.get_biome(pos);
                    let block = biome.get_block(depth);
                    chunk.grid[(u,v,w)].id = block;
                }
            }
        }
        Some(chunk)
    }
    
    pub(super) fn unload_chunk(&self, chunk_coord: (i32, i32, i32), chunk: &PhysicalChunk) {
        // TODO implement unloading chunk with changes saved
    }
}