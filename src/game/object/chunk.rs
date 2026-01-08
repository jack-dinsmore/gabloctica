use cgmath::{Matrix3, Vector3, Zero};
use rustc_hash::FxHashMap;

use crate::{graphics::{CHUNK_SIZE, Camera, CubeGrid, Graphics, ModelUniform, StorageBuffer}, physics::RigidBody};

pub(super) struct Chunk {
    pub(super) grid: CubeGrid,
    pub mass_m0: f64,
    pub mass_m1: Vector3<f64>,
    pub mass_m2: Matrix3<f64>,
    /// 0_0_z-_z+_y-_y+_x-_x+. One means transparent.
    pub flush: u8,
    /// 0_0_z-_z+_y-_y+_x-_x+. One means hidden.
    pub exposed: u8,
    /// 0_0_z-_z+_y-_y+_x-_x+. One means hidden.
    pub detail: usize,
}

impl Chunk {
    pub fn empty(graphics: &Graphics, pos: Vector3<f32>) -> Self {
        let mut grid = CubeGrid::new(graphics);
        grid.global_pos = pos;
        Self {
            grid,
            mass_m0: 0.,
            mass_m1: Vector3::zero(),
            mass_m2: Matrix3::zero(),
            flush: 0,
            detail: 1,
            exposed: 0xff, // Ensures that the chunk model will be updated next frame
        }
    }

    // Update the rigid body collider to have the current block layout.
    pub fn update_metadata(&mut self, blocks: &mut [u16; (CHUNK_SIZE*CHUNK_SIZE) as usize]) {
        // Update the collider
        self.mass_m0 = 0.;
        self.mass_m1 = Vector3::zero();
        self.mass_m2 = Matrix3::zero();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let mut block: u16 = 0;
                for x in 0..CHUNK_SIZE {
                    if self.grid[(x,y,z)] != 0 {
                        block |= 1 << x;

                        let block_mass = 1.;
                        self.mass_m0 += block_mass;
                        self.mass_m1.x += x as f64 * block_mass;
                        self.mass_m1.y += y as f64 * block_mass;
                        self.mass_m1.z += z as f64 * block_mass;
                        self.mass_m2.x[0]+= (x*x) as f64 * block_mass;
                        self.mass_m2.x[1] += (x*y) as f64 * block_mass;
                        self.mass_m2.x[2] += (x*z) as f64 * block_mass;
                        self.mass_m2.y[0] += (y*x) as f64 * block_mass;
                        self.mass_m2.y[1] += (y*y) as f64 * block_mass;
                        self.mass_m2.y[2] += (y*z) as f64 * block_mass;
                        self.mass_m2.z[0] += (z*x) as f64 * block_mass;
                        self.mass_m2.z[1] += (z*y) as f64 * block_mass;
                        self.mass_m2.z[2] += (z*z) as f64 * block_mass;
                    }
                }
                blocks[(y+CHUNK_SIZE*z) as usize] = block;
            }
        }

        self.updated_flush();
    }

    fn updated_flush(&mut self) {
        self.flush = 0;
        let mut break_pos = false;
        let mut break_neg = false;
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                if self.grid[(0,y,z)] == 0 {
                    self.flush |= 1 << 0;
                    break_pos = true;
                }
                if self.grid[(CHUNK_SIZE-1,y,z)] == 0 {
                    self.flush |= 1 << 1;
                    break_neg = true;
                }
                if break_neg && break_pos {
                    break;
                }
            }
        }
        let mut break_pos = false;
        let mut break_neg = false;
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if self.grid[(x,0,z)] == 0 {
                    self.flush |= 1 << 2;
                    break_pos = true;
                }
                if self.grid[(x,CHUNK_SIZE-1,z)] == 0 {
                    self.flush |= 1 << 3;
                    break_neg = true;
                }
                if break_neg && break_pos {
                    break;
                }
            }
        }
        let mut break_pos = false;
        let mut break_neg = false;
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if self.grid[(x,y,0)] == 0 {
                    self.flush |= 1 << 4;
                    break_pos = true;
                }
                if self.grid[(x,y,CHUNK_SIZE-1)] == 0 {
                    self.flush |= 1 << 5;
                    break_neg = true;
                }
                if break_neg && break_pos {
                    break;
                }
            }
        }
    }

    /// Update the exposed flag. Return true if it was changed
    pub fn update_exposed(my_coord: (i32, i32, i32), chunks: &FxHashMap<(i32, i32, i32), Chunk>, exposed: &mut FxHashMap<(i32, i32, i32), u8>) {
        let mut new_exposed = 0;
        let fore = (my_coord.0+1, my_coord.1, my_coord.2);
        let back = (my_coord.0-1, my_coord.1, my_coord.2);
        let left = (my_coord.0, my_coord.1+1, my_coord.2);
        let right = (my_coord.0, my_coord.1-1, my_coord.2);
        let up = (my_coord.0, my_coord.1, my_coord.2+1);
        let down = (my_coord.0, my_coord.1, my_coord.2-1);
        if let Some(c) = chunks.get(&fore) {
            if (c.flush & (1<<1)) == 0 {
                new_exposed |= 1 << 0;
            }
        }
        if let Some(c) = chunks.get(&back) {
            if (c.flush & (1<<0)) == 0 {
                new_exposed |= 1 << 1;
            }
        }
        if let Some(c) = chunks.get(&left) {
            if (c.flush & (1<<3)) == 0 {
                new_exposed |= 1 << 2;
            }
        }
        if let Some(c) = chunks.get(&right) {
            if (c.flush & (1<<2)) == 0 {
                new_exposed |= 1 << 3;
            }
        }
        if let Some(c) = chunks.get(&up) {
            if (c.flush & (1<<5)) == 0 {
                new_exposed |= 1 << 4;
            }
        }
        if let Some(c) = chunks.get(&down) {
            if (c.flush & (1<<4)) == 0 {
                new_exposed |= 1 << 5;
            }
        }

        exposed.insert(my_coord, new_exposed);
    }
    
    /// Update the model buffer of the grid
    pub(crate) fn update_model(&mut self, graphics: &Graphics) {
        self.grid.update_model(graphics, self.detail, self.exposed);
    }

    /// Update the graphics buffer in the grid from the Rigid body.
    pub fn get_uniform(&mut self, body: &RigidBody, camera: &Camera) -> ModelUniform {
        self.grid.get_uniform(
            (body.pos - body.ori * body.com_pos).cast().unwrap(),
            body.ori.cast().unwrap(),
            camera
        )
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        self.grid.draw(render_pass);
    }
    
    pub(crate) fn copy_buffer(&self, encoder: &mut wgpu::CommandEncoder, buffer: &StorageBuffer, index:u32) {
        self.grid.buffer.copy_from_storage_buffer(encoder, buffer, index as u64 *std::mem::size_of::<ModelUniform>() as u64);
    }
}