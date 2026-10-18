
pub mod chunk;
pub mod loader;
pub mod computer;
mod internals;
mod blocks;

use cgmath::{InnerSpace, Matrix3, Quaternion, Rotation, Vector3, Zero};
use loader::{PlanetLoader, ShipLoader};
use rustc_hash::FxHashMap;
use crate::game::object::chunk::PhysicalBlock;
pub use crate::game::object::internals::{Internals, Interrupt};
use crate::graphics::{GraphicalBlock, GraphicalChunk, Graphics, GridTexture, ModelUniform, Renderer, StorageBuffer};
use crate::physics::{Collider, MoI, Physics, RigidBody, RigidBodyInit, RigidBodyData};
use crate::util::{CHUNK_SIZE, my_fmod, moi_product};
use chunk::PhysicalChunk;
pub use internals::BlockGlobalPos;
pub use blocks::{BlockId, GraphicId, BlockProperties};


const RENDER_DISTANCE: i32 = 12; // Units of chunks
const LOAD_TIME: u128 = 25000; // Millseconds

pub enum ObjectLoader {
    OneShot(ShipLoader),
    MultiShot(PlanetLoader),
}
impl ObjectLoader {
    fn estimate_max_rendered_chunks(&self) -> usize {
        8184//TODO
    }

    fn get_initial_data(&self) -> RigidBodyInit {
        match self {
            ObjectLoader::OneShot(ship_loader) => RigidBodyInit {
                pos: ship_loader.pos,
                vel: ship_loader.vel,
                ori: Quaternion::new(0.5, 0.5, 0.5, 1.).normalize(),
                ang_vel: Vector3::new(0., 0., 0.5),
                mass: 0.,
                ..Default::default()
            },
            ObjectLoader::MultiShot(planet_loader) => {
                let mass = 1000.;
                RigidBodyInit {
                    mass,
                    moi: MoI::new_diagonal(Vector3::new(mass, mass, mass)*1000.),
                    ..Default::default()
                }
            },
        }
    }
}

pub struct Object {
    physical_chunks: FxHashMap<(i32, i32, i32), PhysicalChunk>,
    graphical_chunks: FxHashMap<(i32, i32, i32), GraphicalChunk>,
    internals: Internals,
    pub body: RigidBody,
    
    last_load: Option<std::time::Instant>,
    
    loader: ObjectLoader,
    storage_buffer: StorageBuffer,
}

impl Object {
    pub fn new(graphics: &Graphics, physics: &mut Physics, loader: ObjectLoader) -> Self {
        let mut initial_data = loader.get_initial_data();
        initial_data.collider = Some(Collider::empty_object());
        let body = RigidBodyData::new(&mut physics.rb_vendor, initial_data);
        let buffer_size = loader.estimate_max_rendered_chunks()*std::mem::size_of::<ModelUniform>();
        let storage_buffer = StorageBuffer::new(graphics, buffer_size as usize);
        let internals = Internals::new();
        let mut object = Self {
            physical_chunks: FxHashMap::default(),
            graphical_chunks: FxHashMap::default(),
            loader,
            internals,
            body,
            last_load: None,
            storage_buffer,
        };
        object.recalc_rigid_body();
        object
    }

    pub fn update(&mut self, graphics: &Graphics, properties: &BlockProperties, character_pos: Vector3<f64>, delta_t: f64) {
        self.load_chunks(graphics, properties, character_pos);
        self.internals.update(delta_t);
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
    
    /// Insert a block into the cell containg position pos. Pos is in body coordinates.
    pub(crate) fn insert_block(&mut self, graphics: &Graphics, properties: &BlockProperties, typ: BlockId, pos: Vector3<f64>) {
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
    }


    /// Update the model buffers and rigid body. If only some chunks were changed, pass a vector of chunk positions. Otherwise, pass an empty vector. This will update the passed chunks, and neighbors if the neighbors now become visible.
    fn recalc_rigid_body(&mut self) {
        // Determine if the body can be recalculated
        if let ObjectLoader::MultiShot(..) = self.loader {
            return;
        }

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

    fn load_chunks(&mut self, graphics: &Graphics, properties: &BlockProperties, character_pos: Vector3<f64>) {
        if let Some(l) = self.last_load {
            if l.elapsed().as_millis() < LOAD_TIME {
                return;
            }
        }

        // Get the current chunk
        let pos_body = self.body.ori.invert() * (character_pos - self.body.pos);
        let character_chunk = (
            pos_body.x as i32 / CHUNK_SIZE as i32,
            pos_body.y as i32 / CHUNK_SIZE as i32,
            pos_body.z as i32 / CHUNK_SIZE as i32
        );

        match &self.loader {
            ObjectLoader::OneShot(l) => { // e.g. ships
                if self.physical_chunks.is_empty() {
                    let dist = (character_pos - self.body.pos).magnitude();
                    let collider = self.body.get_object_collider_mut();
                    if dist < (RENDER_DISTANCE * CHUNK_SIZE as i32) as f64 * 1.5 {
                        // Load
                        self.physical_chunks = l.load_all(graphics, properties);
                        for (chunk_pos, physical_chunk) in &mut self.physical_chunks {
                            physical_chunk.update_collider(*chunk_pos, &mut self.body);
                            self.graphical_chunks.insert(*chunk_pos, GraphicalChunk::from_physical(graphics, *chunk_pos, physical_chunk, properties));
                        }
                        self.recalc_rigid_body();
                    }
                } else {
                    // Check if all chunks are outside render distance
                    for chunk_coord in self.physical_chunks.keys() {
                        if (chunk_coord.0 - character_chunk.0).abs() < RENDER_DISTANCE
                        && (chunk_coord.1 - character_chunk.1).abs() < RENDER_DISTANCE
                        && (chunk_coord.2 - character_chunk.2).abs() < RENDER_DISTANCE {
                            // One is inside. Therefore I should not unload the object.
                            return;
                        }
                    }
        
                    // Unload
                    l.unload_all();
                    self.physical_chunks.clear();
                    self.graphical_chunks.clear();
                    // TODO delete myself
                }
            },
            ObjectLoader::MultiShot(l) => { // e.g. planets
                let collider = self.body.get_object_collider_mut();
                // Unload old chunks
                let keys = self.graphical_chunks.keys().copied().collect::<Vec<_>>();
                for coord in keys {
                    let detail = get_detail(coord.0 - character_chunk.0, coord.1 - character_chunk.1,coord.2 - character_chunk.2,);
                    if detail == 0 {
                        l.unload_chunk(coord, &self.physical_chunks[&coord]);
                        self.physical_chunks.remove(&coord);
                        self.graphical_chunks.remove(&coord);
                        collider.chunks.remove(&coord);
                    } else {
                        self.graphical_chunks.get_mut(&coord).unwrap().set_detail(detail, graphics);
                    }
                }

                // Load new chunks
                for dx in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                    for dy in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                        for dz in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                            let coord = (character_chunk.0 + dx, character_chunk.1 + dy, character_chunk.2 + dz);
                            if let None = self.physical_chunks.get(&coord) {
                                if let Some(mut chunk) = l.load_chunk(graphics, properties, coord) {
                                    chunk.update_collider(coord, &mut self.body);
                                    let mut graphical_chunk = GraphicalChunk::from_physical(graphics, coord, &chunk, properties);
                                    graphical_chunk.set_detail(get_detail(dx, dy, dz), graphics);
                                    self.graphical_chunks.insert(coord, graphical_chunk);
                                    self.physical_chunks.insert(coord, chunk);
                                }
                            }
                        }
                    }
                }

                self.recalc_rigid_body();
                self.last_load = Some(std::time::Instant::now());
            },
        }
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
    
    pub(crate) fn copy_buffers(&self, renderer: &mut Renderer) {
        let mut i = 0;
        for chunk in self.graphical_chunks.values() {
            if chunk.exposed != 63  {
                chunk.copy_buffer(renderer, &self.storage_buffer, i);
                i += 1;
            }
        }
    }
}

/// Return the detail of the cube given how far away it is
fn get_detail(dx: i32, dy: i32, dz: i32) -> usize {
    let dist = dx.abs().max(dy.abs().max(dz.abs()));
    match dist {
        0..8 => 1,
        8..12 => 2,
        // 12..RENDER_DISTANCE => 4,
        _ => 0, // Unloaded
    }
}