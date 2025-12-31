use cgmath::{Matrix3, Vector3, Zero};

use crate::{graphics::{CHUNK_SIZE, Camera, CubeGrid, Graphics}, physics::RigidBody};

pub(super) struct Chunk {
    pub(super) grid: CubeGrid,
    pub mass_m0: f64,
    pub mass_m1: Vector3<f64>,
    pub mass_m2: Matrix3<f64>,
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
        }
    }

    // Update the rigid body collider to have the current block layout.
    pub fn update_rigid_body(&mut self, blocks: &mut [u16; (CHUNK_SIZE*CHUNK_SIZE) as usize]) {
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
    }

    /// Update the graphics buffer in the grid from the Rigid body.
    pub fn update_buffer(&mut self, body: &RigidBody, graphics: &Graphics, camera: &Camera) {
        self.grid.update_buffer(
            graphics,
            (body.pos - body.ori * body.com_pos).cast().unwrap(),
            body.ori.cast().unwrap(),
            camera
        );
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        self.grid.draw(render_pass);
    }
}