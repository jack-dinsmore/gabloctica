
mod ship;
mod planet;
pub mod chunk;
pub mod computer;
mod internals;
mod blocks;

use cgmath::{Matrix3, Vector3, Zero};
use rustc_hash::FxHashMap;
use crate::graphics::{GraphicalBlock, GraphicalChunk, Graphics, GridTexture, ModelUniform, Renderer, StorageBuffer};
use crate::physics::{Collider, MoI, Physics, RigidBody, RigidBodyInit, RigidBodyData};
use crate::util::{CHUNK_SIZE, my_fmod, moi_product};
use chunk::{PhysicalChunk, PhysicalBlock};

pub use internals::{Internals, Interrupt, BlockGlobalPos};
pub use blocks::{BlockId, GraphicId, BlockProperties};
pub use planet::Planet;
pub use ship::Ship;

pub const RENDER_DISTANCE: i32 = 12; // Units of chunks

#[derive(Debug, Clone, Copy)]
pub enum ObjectIdentifier {
    Ship(usize),
    Planet(usize),
}
impl ObjectIdentifier {
    pub fn get_object<'a>(&self, ships: &'a [Ship], planets: &'a [Planet]) -> &'a Object {
        match self {
            ObjectIdentifier::Ship(i) => &ships[*i].object,
            ObjectIdentifier::Planet(i) => &planets[*i].object,
        }
    }
    pub fn get_object_mut<'a>(&self, ships: &'a mut [Ship], planets: &'a mut [Planet]) -> &'a mut Object {
        match self {
            ObjectIdentifier::Ship(i) => &mut ships[*i].object,
            ObjectIdentifier::Planet(i) => &mut planets[*i].object,
        }
    }
}

pub struct Object {
    pub physical_chunks: FxHashMap<(i32, i32, i32), PhysicalChunk>,
    pub graphical_chunks: FxHashMap<(i32, i32, i32), GraphicalChunk>,
    internals: Internals,
    pub body: RigidBody,
    storage_buffer: StorageBuffer,
}

impl Object {
    pub fn new(graphics: &Graphics, physics: &mut Physics, mut initial_data: RigidBodyInit) -> Self {
        initial_data.collider = Some(Collider::empty_object());
        let body = RigidBodyData::new(&mut physics.rb_vendor, initial_data);
        let buffer_size = 8184*std::mem::size_of::<ModelUniform>(); // TODO
        let storage_buffer = StorageBuffer::new(graphics, buffer_size as usize);
        let internals = Internals::new();
        let mut object = Self {
            physical_chunks: FxHashMap::default(),
            graphical_chunks: FxHashMap::default(),
            internals,
            body,
            storage_buffer,
        };
        object.recalc_rigid_body();
        object
    }

    /// Interact with a block through right clicking
    pub fn interrupt(&mut self, block: BlockGlobalPos, interrupt: Interrupt) {
        self.internals.interrupt(block, interrupt);
    }

    pub fn get_block(&self, pos: Vector3<f64>) -> &PhysicalBlock {
        let updated_chunk = (
            (pos.x/CHUNK_SIZE as f64).floor() as i32,
            (pos.y/CHUNK_SIZE as f64).floor() as i32,
            (pos.z/CHUNK_SIZE as f64).floor() as i32,
        );
        let updated_block = (
            my_fmod(pos.x, CHUNK_SIZE as f64) as u32,
            my_fmod(pos.y, CHUNK_SIZE as f64) as u32,
            my_fmod(pos.z, CHUNK_SIZE as f64) as u32,
        );

        let chunk = self.physical_chunks.get(&updated_chunk).unwrap();
        &chunk.grid[updated_block]
    }

    fn update(&mut self, delta_t: f64) {
        self.internals.update(delta_t);
    }
    
    /// Insert a block into the cell containg position pos. Pos is in body coordinates.
    pub fn insert_block(&mut self, graphics: &Graphics, properties: &BlockProperties, typ: BlockId, pos: Vector3<f64>) {
        let block_pos = ((
            (pos.x/CHUNK_SIZE as f64).floor() as i32,
            (pos.y/CHUNK_SIZE as f64).floor() as i32,
            (pos.z/CHUNK_SIZE as f64).floor() as i32,
        ), (
            my_fmod(pos.x, CHUNK_SIZE as f64) as u32,
            my_fmod(pos.y, CHUNK_SIZE as f64) as u32,
            my_fmod(pos.z, CHUNK_SIZE as f64) as u32,
        ));

        if let None = self.physical_chunks.get(&block_pos.0) {
            // Make a new chunk
            let pos = Vector3::new(block_pos.0.0 as f64, block_pos.0.1 as f64, block_pos.0.2 as f64)*CHUNK_SIZE as f64;

            let new_chunk = PhysicalChunk::empty(graphics, block_pos.0);
            self.physical_chunks.insert(block_pos.0, new_chunk);

            let new_chunk = GraphicalChunk::empty(graphics, block_pos.0);
            self.graphical_chunks.insert(block_pos.0, new_chunk);

            self.body.get_object_collider_mut().chunks.insert(block_pos.0, [0; (CHUNK_SIZE*CHUNK_SIZE) as usize]);
        }
        // Set the block
        let block = PhysicalBlock::new(typ, 0, properties);
        self.internals.place_block(&block, properties, &self.physical_chunks);
        self.physical_chunks.get_mut(&block_pos.0).unwrap().place_block(block, block_pos, &mut self.body);

        let block = GraphicalBlock::new(properties[typ].graphic_id);
        self.graphical_chunks.get_mut(&block_pos.0).unwrap().place_block(block, block_pos, graphics);
        GraphicalChunk::update_exposed(block_pos.0, &mut self.graphical_chunks, graphics);

        // TODO why not recalc_rigid_body?
    }


    /// Update the model buffers and rigid body. If only some chunks were changed, pass a vector of chunk positions. Otherwise, pass an empty vector. This will update the passed chunks, and neighbors if the neighbors now become visible.
    fn recalc_rigid_body(&mut self) {
        let mut mass_m0 = 0.;
        let mut mass_m1 = Vector3::zero();
        let mut mass_m2 = Matrix3::zero();
        for (chunk_pos, chunk) in &self.physical_chunks {
            let chunk_pos = Vector3::new(chunk_pos.0 as f64, chunk_pos.1 as f64, chunk_pos.2 as f64) * CHUNK_SIZE as f64;
            mass_m0 += chunk.mass_m0;
            mass_m1 += chunk.mass_m1 + chunk_pos * chunk.mass_m0;
            mass_m2 += chunk.mass_m2 + moi_product(chunk_pos) * chunk.mass_m0;
        }
        let first_initialization = self.body.mass == 0. && mass_m0 != 0.;
        self.body.mass = mass_m0;
        mass_m1 /= mass_m0;

        // If the body has not been loaded, stop.
        if mass_m0 == 0. {return;}

        // Shift the position to preserve com
        let delta_com_global = self.body.ori * (mass_m1 - self.body.com_pos);
        if !first_initialization {
            self.body.pos += delta_com_global;
        }

        // Center of mass
        mass_m2 -= moi_product(mass_m1) * mass_m0;
        self.body.com_pos = mass_m1;

        // MOI
        self.body.moi = MoI::new_matrix(mass_m2);
    }
}

// Graphics
impl Object {
    pub fn update_buffer(&mut self, graphics: &Graphics, camera: &crate::graphics::Camera) {
        let mut buffer = Vec::with_capacity(self.physical_chunks.len());
        for chunk in self.graphical_chunks.values_mut() {
            if chunk.exposed != 63  {
                buffer.push(chunk.get_uniform(&self.body, camera));
            }
        }
        self.storage_buffer.write(graphics, &buffer);
    }

    /// Draw for the sake of the shadow buffer (in which case the texture need not be bound)
    pub fn draw(&self, renderer: &mut Renderer, texture: &GridTexture) {
        for detail in 1..=4 {
            texture.bind(renderer, detail);
            for chunk in self.graphical_chunks.values() {
                if chunk.exposed != 63 && chunk.detail == detail {
                    chunk.draw(renderer);
                }
            }
        }
    }
    
    /// Draw for the sake of the shadow buffer (in which case the texture need not be bound)
    pub fn draw_shadow(&self, renderer: &mut Renderer) {
        for detail in 1..=4 {
            for chunk in self.graphical_chunks.values() {
                if chunk.exposed != 63 && chunk.detail == detail {
                    chunk.draw(renderer);
                }
            }
        }
    }
    
    pub fn copy_buffers(&self, renderer: &mut Renderer) {
        let mut i = 0;
        for chunk in self.graphical_chunks.values() {
            if chunk.exposed != 63  {
                chunk.copy_buffer(renderer, &self.storage_buffer, i);
                i += 1;
            }
        }
    }
}