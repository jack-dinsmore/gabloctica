use cgmath::{InnerSpace, Matrix3, Rotation, Vector3, Zero};

use loader::{PlanetLoader, ShipLoader};
use rustc_hash::FxHashMap;
use crate::graphics::{CHUNK_SIZE, Graphics, GridTexture, ModelUniform, StorageBuffer};
use crate::physics::{Collider, Physics, RigidBody, RigidBodyInit};
pub use planet::{Planet, PlanetInit};

mod chunk;
mod planet;
mod loader;
use chunk::Chunk;

const RENDER_DISTANCE: i32 = 12; // Units of chunks
const LOAD_TIME: u128 = 250; // Millseconds

pub enum ObjectLoader {
    OneShot(ShipLoader),
    MultiShot(PlanetLoader),
}
impl ObjectLoader {
    fn estimate_max_rendered_chunks(&self) -> usize {
        8184//TODO
    }
}

pub struct Object {
    chunks: FxHashMap<(i32, i32, i32), Chunk>,
    loader: ObjectLoader,
    pub body: RigidBody,
    last_load: std::time::Instant,
    storage_buffer: StorageBuffer,
}
impl Object {
    pub fn new(graphics: &Graphics, physics: &mut Physics, loader: ObjectLoader, character_pos: Vector3<f64>) -> Self {
        let initial_data = RigidBodyInit {
            collider: Some(Collider::empty_object()),
            ..Default::default()
        };
        let body = RigidBody::new(physics, initial_data);
        let buffer_size = loader.estimate_max_rendered_chunks()*std::mem::size_of::<ModelUniform>();
        let storage_buffer = StorageBuffer::new(graphics, buffer_size as usize);
        let mut out = Self {
            chunks: FxHashMap::default(),
            loader: loader,
            body,
            last_load: std::time::Instant::now(),
            storage_buffer,
        };
        out.load_chunks(graphics, character_pos);
        out
    }

    /// Update the model buffers and rigid body. If only some chunks were changed, pass a vector of chunk positions. Otherwise, pass an empty vector. This will update the passed chunks, and neighbors if the neighbors now become visible.
    pub fn update_chunk_info(&mut self, graphics: &Graphics, coord_vec: Vec<(i32, i32, i32)>) {
        let mut mass_m0 = 0.;
        let mut mass_m1 = Vector3::zero();
        let mut mass_m2 = Matrix3::zero();
        let collider = self.body.get_object_collider_mut();
        if coord_vec.is_empty() {
            for (coord, chunk) in &mut self.chunks {
                chunk.update_metadata(collider.chunks.get_mut(coord).unwrap());
                mass_m0 += chunk.mass_m0;
                mass_m1 += chunk.mass_m1;
                mass_m2 += chunk.mass_m2;
            }
        } else {
            for coord in coord_vec.iter() {
                self.chunks.get_mut(coord).unwrap().update_metadata(collider.chunks.get_mut(coord).unwrap());
                mass_m0 += self.chunks[coord].mass_m0;
                mass_m1 += self.chunks[coord].mass_m1;
                mass_m2 += self.chunks[coord].mass_m2;
            }
        }

        // Update the buffers
        let mut exposed_map = FxHashMap::default();
        if coord_vec.is_empty() {
            for coord in self.chunks.keys() {
                Chunk::update_exposed(*coord, &self.chunks, &mut exposed_map);
            }
        } else {
            for coord in coord_vec.iter() {
                Chunk::update_exposed(*coord, &self.chunks, &mut exposed_map);
            }
        }
        for (coord, chunk) in &mut self.chunks {
            if let Some(new_exposed) = exposed_map.get(coord) {
                if *new_exposed != chunk.exposed {
                    chunk.exposed = *new_exposed;
                    chunk.update_model(graphics);
                }
            }
        }

        // Set the rigid body data
        mass_m1 /= mass_m0;
        mass_m2 /= mass_m0;
        self.body.com_pos = mass_m1.cast().unwrap(); // Center of mass
        self.body.moi = crate::physics::MoI::new_matrix((mass_m2.clone() - Matrix3::new(
            mass_m1.x*mass_m1.x - 0.1666666666, mass_m1.x*mass_m1.y, mass_m1.x*mass_m1.z,
            mass_m1.y*mass_m1.x, mass_m1.y*mass_m1.y - 0.1666666666, mass_m1.y*mass_m1.z,
            mass_m1.z*mass_m1.x, mass_m1.z*mass_m1.y, mass_m1.z*mass_m1.z - 0.1666666666,
        )) * mass_m0);
    }

    pub fn update(&mut self, graphics: &Graphics, character_pos: Vector3<f64>) {
        self.load_chunks(graphics, character_pos);
    }

    fn load_chunks(&mut self, graphics: &Graphics, character_pos: Vector3<f64>) {
        if self.last_load.elapsed().as_millis() < LOAD_TIME {return;}

        // Get the current chunk
        let pos_body = self.body.ori.invert() * (character_pos - self.body.pos);
        let character_chunk = (
            pos_body.x as i32 / CHUNK_SIZE as i32,
            pos_body.y as i32 / CHUNK_SIZE as i32,
            pos_body.z as i32 / CHUNK_SIZE as i32
        );

        match &self.loader {
            ObjectLoader::OneShot(l) => {
                if self.chunks.is_empty() {
                    let dist = (character_pos - self.body.pos).magnitude();
                    let collider = self.body.get_object_collider_mut();
                    if dist < (RENDER_DISTANCE * CHUNK_SIZE as i32) as f64 * 1.5 {
                        self.chunks = l.load_all(graphics);
                        for coord in self.chunks.keys() {
                            collider.chunks.insert(*coord, [0; (CHUNK_SIZE*CHUNK_SIZE) as usize]);
                        }
                        self.update_chunk_info(graphics, Vec::new());
                    }
                } else {
                    // Check if all chunks are outside render distance
                    for chunk_coord in self.chunks.keys() {
                        if (chunk_coord.0 - character_chunk.0).abs() < RENDER_DISTANCE
                        && (chunk_coord.1 - character_chunk.1).abs() < RENDER_DISTANCE
                        && (chunk_coord.2 - character_chunk.2).abs() < RENDER_DISTANCE {
                            // One is inside. Therefore I should not unload the object.
                            return;
                        }
                    }
        
                    // Unload everything
                    l.unload_all();
                    self.chunks.clear();
                    let collider = self.body.get_object_collider_mut();
                    collider.chunks.clear();
                }
            },
            ObjectLoader::MultiShot(l) => {
                let collider = self.body.get_object_collider_mut();
                // Unload old chunks
                let mut delete_coords = Vec::new();
                for (coord, chunk) in &mut self.chunks {
                    let detail = get_detail(coord.0 - character_chunk.0, coord.1 - character_chunk.1,coord.2 - character_chunk.2,);
                    if detail == 0 {
                        l.unload_chunk(*coord, chunk);
                        delete_coords.push(*coord);
                    } else if detail != chunk.detail {
                        chunk.detail = detail;
                        chunk.update_model(graphics); // Update the model with the new detail
                    }
                }
                for coord in delete_coords.into_iter().rev() {
                    self.chunks.remove(&coord);
                    collider.chunks.remove(&coord);
                }

                // Load new chunks
                let mut new_coords = Vec::new();
                for dx in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                    for dy in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                        for dz in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                            let coord = (character_chunk.0 + dx, character_chunk.1 + dy, character_chunk.2 + dz);
                            if let None = self.chunks.get(&coord) {
                                if let Some(mut chunk) = l.load_chunk(graphics, coord) {
                                    chunk.detail = get_detail(dx, dy, dz);
                                    self.chunks.insert(coord, chunk);
                                    collider.chunks.insert(coord, [0; (CHUNK_SIZE*CHUNK_SIZE) as usize]);
                                    new_coords.push(coord);
                                }
                            }
                        }
                    }
                }
                if !new_coords.is_empty() {
                    self.update_chunk_info(graphics, new_coords);
                }
                self.last_load = std::time::Instant::now();
            },
        }
    }
    
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>, texture: &GridTexture) {
        for detail in 1..=4 {
            texture.bind(render_pass, detail);
            for chunk in self.chunks.values() {
                if chunk.exposed != 63 && chunk.detail == detail {
                    chunk.draw(render_pass);
                }
            }
        }
    }
    
    pub fn update_buffer(&mut self, graphics: &Graphics, camera: &crate::graphics::Camera) {
        let mut buffer = Vec::with_capacity(self.chunks.len());
        for chunk in self.chunks.values_mut() {
            if chunk.exposed != 63  {
                buffer.push(chunk.get_uniform(&self.body, camera));
            }
        }
        self.storage_buffer.write(graphics, buffer);
    }
    
    pub(crate) fn copy_buffers(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut i = 0;
        for chunk in self.chunks.values() {
            if chunk.exposed != 63  {
                chunk.copy_buffer(encoder, &self.storage_buffer, i);
                i += 1;
            }
        }
    }
    
    /// Insert a block into the cell containg position pos. Pos is in body coordinates.
    pub(crate) fn insert_block(&mut self, graphics: &Graphics, typ: u16, pos: Vector3<f64>) {
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

        if let None = self.chunks.get(&updated_chunk) {
            // Make a new chunk
            let pos = Vector3::new(updated_chunk.0 as f32, updated_chunk.1 as f32, updated_chunk.2 as f32)*CHUNK_SIZE as f32;
            let new_chunk = Chunk::empty(graphics, pos);
            self.chunks.insert(updated_chunk, new_chunk);
            self.body.get_object_collider_mut().chunks.insert(updated_chunk, [0; (CHUNK_SIZE*CHUNK_SIZE) as usize]);
        }
        // Set the block
        let chunk = self.chunks.get_mut(&updated_chunk).unwrap();
        chunk.grid[updated_block] = typ;
        chunk.update_model(graphics);
        self.update_chunk_info(graphics, vec![updated_chunk]);
    }
}

fn my_fmod(f: f64, l: f64) -> f64 {
    let phase = f / l;
    (phase - phase.floor()) * l
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