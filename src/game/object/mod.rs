use cgmath::{InnerSpace, Quaternion, Rad, Rotation, Rotation3, Vector3, Zero};
use faer::Mat;

use loader::{PlanetLoader, ShipLoader};
use crate::graphics::{CHUNK_SIZE, Graphics};
use crate::physics::{Collider, Physics, RigidBody, RigidBodyInit};

mod chunk;
mod loader;
use chunk::Chunk;

const RENDER_DISTANCE: i32 = 4; // Units of chunks
const LOAD_TIME: u128 = 250; // Millseconds

pub enum ObjectLoader {
    OneShot(ShipLoader),
    MultiShot(PlanetLoader),
}
impl ObjectLoader {
    pub fn demo() -> Self {
        Self::MultiShot(PlanetLoader {})
        // Self::OneShot(ShipLoader {})
    }
}

pub struct Object {
    chunks: Vec<Chunk>,
    coords: Vec<(i32, i32, i32)>,
    loader: ObjectLoader,
    pub body: RigidBody,
    last_load: std::time::Instant,
}
impl Object {
    pub fn new(graphics: &Graphics, physics: &mut Physics, loader: ObjectLoader, character_pos: Vector3<f64>) -> Self {
        let initial_data = RigidBodyInit {
            collider: Some(Collider::empty_object()),
            ..Default::default()
        };
        let body = RigidBody::new(physics, initial_data);
        let mut out = Self {
            chunks: Vec::new(),
            coords: Vec::new(),
            loader: loader,
            body,
            last_load: std::time::Instant::now(),
        };
        out.load_chunks(graphics, character_pos);
        out
    }

    pub fn update_rigid_body(&mut self, chunk_pos: (i32, i32, i32)) {
        let mut mass_m0 = 0.;
        let mut mass_m1 = Vector3::zero();
        let mut mass_m2 = Mat::zeros(3,3);
        let collider = self.body.get_object_collider_mut();
        for (i, coord) in self.coords.iter().enumerate() {
            if *coord == chunk_pos {
                self.chunks[i].update_rigid_body(&mut collider.chunks[i]);
            }
            mass_m0 += self.chunks[i].mass_m0;
            mass_m1 += self.chunks[i].mass_m1;
            mass_m2 += &self.chunks[i].mass_m2;
        }

        // Set the rigid body data
        mass_m1 /= mass_m0;
        mass_m2 /= mass_m0;
        let moi = (mass_m2.clone() - faer::mat![
            [mass_m1.x*mass_m1.x + 0.1666666666, mass_m1.x*mass_m1.y, mass_m1.x*mass_m1.z],
            [mass_m1.y*mass_m1.x, mass_m1.y*mass_m1.y + 0.1666666666, mass_m1.y*mass_m1.z],
            [mass_m1.z*mass_m1.x, mass_m1.z*mass_m1.y, mass_m1.z*mass_m1.z + 0.1666666666],
        ]) * mass_m0;

        // TODO ensure that the block doesn't get rotated by a switch in the order of eivengectors
        let eigen = moi.eigen().unwrap();
        let evecs = eigen.U();
        let evals = eigen.S();
        let moi = Vector3::new(evals[0].re, evals[1].re, evals[2].re);
        
        let global_ori = {
            // Get the rotation quaternion of the eigenvector matrix, which is a rotation matrix
            let eigen = evecs.eigen().unwrap();
            let rot_evals = eigen.S();
            let args = [rot_evals[0].arg(), rot_evals[1].arg(), rot_evals[2].arg()];
            let mut indices = vec![0, 1, 2];
            indices.sort_by(|a, b| args[*a].partial_cmp(&args[*b]).unwrap());
            let angle = args[indices[2]];
            let axis = Vector3::new(evecs[(indices[1],0)].re, evecs[(indices[1],1)].re, evecs[(indices[1],2)].re);
            Quaternion::from_axis_angle(axis, Rad(angle))
        };
        let global_pos: Vector3<f64> = -mass_m1.cast().unwrap(); // Center of mass

        // Update all the blocks to shift their orientations and positions
        // TODO
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
                if self.coords.is_empty() {
                    let dist = (character_pos - self.body.pos).magnitude();
                    let collider = self.body.get_object_collider_mut();
                    if dist < (RENDER_DISTANCE * CHUNK_SIZE as i32) as f64 * 1.5 {
                        (self.coords, self.chunks) = l.load_all(graphics);
                        for i in 0..self.chunks.len() {
                            collider.chunks.push([0; (CHUNK_SIZE*CHUNK_SIZE) as usize]);
                            collider.coords.push(self.coords[i]);
                            self.chunks[i].update_rigid_body(&mut collider.chunks[i]);
                        }
                    }
                } else {
                    // Check if all chunks are outside render distance
                    for chunk_coord in &self.coords {
                        if (chunk_coord.0 - character_chunk.0).abs() < RENDER_DISTANCE
                        && (chunk_coord.1 - character_chunk.1).abs() < RENDER_DISTANCE
                        && (chunk_coord.2 - character_chunk.2).abs() < RENDER_DISTANCE {
                            // One is inside. Therefore I should not unload the object.
                            return;
                        }
                    }
        
                    // Unload everything
                    l.unload_all();
                    self.coords.clear();
                    self.chunks.clear();
                    let collider = self.body.get_object_collider_mut();
                    collider.coords.clear();
                    collider.chunks.clear();
                }
            },
            ObjectLoader::MultiShot(l) => {
                let collider = self.body.get_object_collider_mut();
                // Unload old chunks
                let mut delete_indices = Vec::new();
                for (i, chunk_coord) in self.coords.iter().enumerate() {
                    if (chunk_coord.0 - character_chunk.0).abs() > RENDER_DISTANCE
                    || (chunk_coord.1 - character_chunk.1).abs() > RENDER_DISTANCE
                    || (chunk_coord.2 - character_chunk.2).abs() > RENDER_DISTANCE {
                        l.unload_chunk(*chunk_coord, &self.chunks[i]);
                        delete_indices.push(i);
                    }
                }
                for j in delete_indices.into_iter().rev() {
                    self.coords.swap_remove(j);
                    self.chunks.swap_remove(j);
                    collider.coords.swap_remove(j);
                    collider.chunks.swap_remove(j);
                }

                // Load new chunks
                for dx in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                    for dy in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                        for dz in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                            let coord = (character_chunk.0 + dx, character_chunk.1 + dy, character_chunk.2 + dz);
                            if self.coords.contains(&coord) { continue; }
                            if let Some(c) = l.load_chunk(graphics, coord) {
                                self.chunks.push(c);
                                self.coords.push(coord);
                                collider.chunks.push([0; (CHUNK_SIZE*CHUNK_SIZE) as usize]);
                                collider.coords.push(coord);
                                let i = self.chunks.len()-1;
                                self.chunks[i].update_rigid_body(&mut collider.chunks[i]);
                            }
                        }
                    }
                }
                self.last_load = std::time::Instant::now();
            },
        }
    }
    
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for chunk in &self.chunks {
            chunk.draw(render_pass);
        }
    }
    
    pub fn update_buffer(&mut self, graphics: &Graphics, camera: &crate::graphics::Camera) {
        for chunk in &mut self.chunks {
            chunk.update_buffer(&self.body, graphics, camera);
        }
    }
    
    /// Insert a block into the cell containg position pos. Pos is in body coordinates.
    pub(crate) fn insert_block(&mut self, graphics: &Graphics, typ: u16, pos: Vector3<f64>) {
        let updated_chunk = (
            pos.x as i32 / CHUNK_SIZE as i32,
            pos.y as i32 / CHUNK_SIZE as i32,
            pos.z as i32 / CHUNK_SIZE as i32,
        );
        let updated_block = (
            (pos.x as i32 % CHUNK_SIZE as i32) as u32,
            (pos.y as i32 % CHUNK_SIZE as i32) as u32,
            (pos.z as i32 % CHUNK_SIZE as i32) as u32,
        );

        let mut found_chunk_index = None;
        for (i, coord) in self.coords.iter().enumerate() {
            if *coord == updated_chunk {
                found_chunk_index = Some(i)
            }
        }
        if let None = found_chunk_index {
            // Make a new chunk
            let pos = Vector3::new(updated_chunk.0 as f32, updated_chunk.1 as f32, updated_chunk.2 as f32);
            let new_chunk = Chunk::empty(graphics, pos);
            self.chunks.push(new_chunk);
            found_chunk_index = Some(self.chunks.len());
        }
        // Set the block
        let chunk = &mut self.chunks[found_chunk_index.unwrap()];
        chunk.grid[updated_block] = typ;

        self.update_rigid_body(updated_chunk);
    }
}