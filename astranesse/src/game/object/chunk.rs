use cgmath::{Matrix3, Vector3, Zero};

use crate::{game::object::{BlockGlobalPos, BlockId, BlockProperties}, graphics::Graphics, physics::RigidBody, util::{CHUNK_SIZE, Grid, moi_product}};

#[derive(Copy, Clone, Debug)]
pub struct PhysicalBlock {
    pub id: BlockId,
    pub ori: u8,
    pub mass: f64,
}
impl PhysicalBlock {
    pub fn new(id: BlockId, ori: u8, properties: &BlockProperties) -> Self {
        Self {
            id,
            ori,
            mass: 1.,
        }
    }
    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}
impl Default for PhysicalBlock {
    fn default() -> Self {
        Self { id: 0, ori: 0, mass: 0. }
    }
}

pub struct PhysicalChunk {
    pub grid: Grid<PhysicalBlock>,
    pub mass_m0: f64,
    pub mass_m1: Vector3<f64>,
    pub mass_m2: Matrix3<f64>,
}

impl PhysicalChunk {
    pub fn empty(graphics: &Graphics, chunk_pos: (i32, i32, i32)) -> Self {
        Self {
            grid: Grid::new(),
            mass_m0: 0.,
            mass_m1: Vector3::zero(),
            mass_m2: Matrix3::zero(),
        }
    }

    // Update the rigid body collider to have the current block layout.
    pub fn place_block(&mut self, block: PhysicalBlock, block_pos: BlockGlobalPos, body: &mut RigidBody) {
        self.grid[block_pos.1] = block;
        self.update_collider(block_pos.0, body);
    }

    /// Update the object's collider to include this chunk
    pub fn update_collider(&mut self, chunk_pos: (i32, i32, i32), body: &mut RigidBody) {
        self.mass_m0 = 0.;
        self.mass_m1 = Vector3::zero();
        self.mass_m2 = Matrix3::zero();

        let collider = body.get_object_collider_mut();
        let collider_blocks = match collider.chunks.get_mut(&chunk_pos) {
            Some(collider_blocks) => collider_blocks,
            None => {
                collider.chunks.insert(chunk_pos, [0; (CHUNK_SIZE*CHUNK_SIZE) as usize]);
                collider.chunks.get_mut(&chunk_pos).unwrap()
            },
        };
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let mut collider_row: u16 = 0;
                for x in 0..CHUNK_SIZE {
                    let pos = Vector3::new(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5);
                    let grid_block = self.grid[(x,y,z)];
                    if !grid_block.is_null() {
                        collider_row |= 1 << x;

                        self.mass_m0 += grid_block.mass;
                        self.mass_m1 += pos * grid_block.mass;
                        self.mass_m2 += moi_product(pos) * grid_block.mass;
                    }
                }
                collider_blocks[(y+CHUNK_SIZE*z) as usize] = collider_row;
            }
        }
        self.mass_m2 += self.mass_m0 * Matrix3::new(1., 0., 0., 0., 1., 0., 0., 0., 1.,) / 6.;
    }
}