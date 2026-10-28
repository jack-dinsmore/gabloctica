use cgmath::{InnerSpace, Vector3};
use rustc_hash::FxHashMap;
use crate::{game::object::{BlockProperties, Object, RENDER_DISTANCE, chunk::{PhysicalBlock, PhysicalChunk}}, graphics::{GraphicalChunk, Graphics}, physics::{Physics, RigidBodyInit}};

pub struct Ship {
    pub object: Object,
    loaded: Option<()>,
}

impl Ship {
    pub fn new(graphics: &Graphics, physics: &mut Physics, rb_init: RigidBodyInit) -> Self {
        let object = Object::new(graphics, physics, rb_init);
        Self {
            object,
            loaded: None,
        }
    }

    pub fn update(&mut self, graphics: &Graphics, properties: &BlockProperties, character_pos: Vector3<f64>, delta_t: f64) {
        self.object.update(delta_t);

        // Load/unload the entire planet
        let dist = (character_pos - self.object.body.pos).magnitude();
        if self.loaded.is_none() && dist < 2. * RENDER_DISTANCE as f64 {
            self.load(graphics, properties);
        };
        if self.loaded.is_some() && dist > 2. * RENDER_DISTANCE as f64 {
            self.loaded = None;
        };
    }
    
    fn load(&mut self, graphics: &Graphics, properties: &BlockProperties) {
        self.object.physical_chunks = {
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

            self.loaded = Some(());
            out
        };

        for (chunk_pos, physical_chunk) in &mut self.object.physical_chunks {
            physical_chunk.update_collider(*chunk_pos, &mut self.object.body);
            self.object.graphical_chunks.insert(*chunk_pos, GraphicalChunk::from_physical(graphics, *chunk_pos, physical_chunk, properties));
        }
        self.object.recalc_rigid_body();
    }
}
