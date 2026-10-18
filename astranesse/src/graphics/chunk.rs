use cgmath::{Matrix4, Vector3};
use image::{GenericImageView, ImageBuffer, Rgba};
use rustc_hash::FxHashMap;

use crate::game::object::{BlockGlobalPos, BlockProperties};
use crate::game::object::chunk::PhysicalChunk;
use crate::graphics::{GraphicalBlock, BlockVertex, Camera, Graphics, Renderer, ResourceType, Texture};
use crate::graphics::resource::{IndexBuffer, VertexBuffer, UniformBuffer, Uniform};
use crate::physics::RigidBody;
use crate::util::{CHUNK_SIZE, Grid};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    model: [[f32; 4]; 4],
}
impl Uniform for ModelUniform {
    const TYPE: ResourceType = ResourceType::Model;
}
impl ModelUniform {
    pub fn new(mat: Matrix4<f32>) -> Self {
        Self {
            model: mat.into()
        }
    }
}

const VERTEX_CAPACITY: usize = 0x10000;//TODO actually calculate this

pub struct GraphicalChunk {
    grid: Grid<GraphicalBlock>,
    /// Offset of the chunk from the rigid body center (i.e. the center of mass)
    vertex_buffer: VertexBuffer<BlockVertex>,
    index_buffer: IndexBuffer,
    n_indices: u32,
    global_pos: Vector3<f64>,

    pub buffer: UniformBuffer<ModelUniform>,
    pub detail: usize,

    /// 0_0_z-_z+_y-_y+_x-_x+. One means hidden.
    pub exposed: u8,
    /// 0_0_z-_z+_y-_y+_x-_x+. One means transparent.
    pub flush: u8,
}

impl GraphicalChunk {
    pub fn empty(graphics: &Graphics, chunk_pos: (i32, i32, i32)) -> Self {
        let grid = Grid::new();

        let vertex_buffer = VertexBuffer::new(graphics, VERTEX_CAPACITY);
        let index_buffer = IndexBuffer::new(graphics, VERTEX_CAPACITY/4*6);
        let buffer = UniformBuffer::new(graphics);
        let global_pos = Vector3::new(chunk_pos.0 as f64, chunk_pos.1 as f64, chunk_pos.2 as f64) * CHUNK_SIZE as f64;

        Self {
            grid,
            vertex_buffer,
            index_buffer,
            n_indices: 0,
            detail: 1,
            exposed: 0xff, // Ensure the exposure flag will be updated next
            flush: 0, // Ensure the exposure flag will be updated next

            buffer,
            global_pos,
        }
    }

    pub fn from_physical(graphics: &Graphics, chunk_pos: (i32, i32, i32), chunk: &PhysicalChunk, properties: &BlockProperties) -> Self {
        let mut output = Self::empty(graphics, chunk_pos);

        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    output.grid[(i,j,k)] = GraphicalBlock::from_physical(chunk.grid[(i,j,k)], properties);
                }
            }
        }

        output.update_flush();
        output.update_model(graphics);
        output
    }

    pub fn place_block(&mut self, block: GraphicalBlock, block_pos: BlockGlobalPos, graphics: &Graphics) {
        self.grid[block_pos.1] = block;
        self.update_flush();
        self.update_model(graphics);
    }

    // Create vertex and index buffers from the block layout. exposed should be set to one for hidden faces
    fn update_model(&mut self, graphics: &Graphics) {
        // Right now I'm just making vertices for every block, including faces that don't face outwards.
        // OPTIMIZE implement exposed
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u16;
        let detail_skip = 2u32.pow(self.detail as u32-1);
        let last = CHUNK_SIZE - detail_skip;
        if detail_skip == 0 {
            // panic!("{}", self.detail); // TODO remove
            return;// TODO BAD
        }
        for x in (0..CHUNK_SIZE).step_by(detail_skip as usize) {
            for y in (0..CHUNK_SIZE).step_by(detail_skip as usize) {
                for z in (0..CHUNK_SIZE).step_by(detail_skip as usize) {
                    let block = self.grid[(x,y,z)];
                    if block.is_null() {continue;}

                    if (block.id & 0x100) == 0 {
                        add_block(&self.grid, x, y, z, detail_skip, &mut index_offset, &mut vertices, &mut indices);
                    } else {
                        add_clutter(&self.grid, x, y, z, detail_skip, &mut index_offset, &mut vertices, &mut indices);
                    }
                }
            }
        }
        if vertices.len() > VERTEX_CAPACITY {
            panic!("too many vertices");
        }

        graphics.queue.write_buffer(&self.vertex_buffer.buffer, 0, bytemuck::cast_slice(&vertices));
        graphics.queue.write_buffer(&self.index_buffer.buffer, 0, bytemuck::cast_slice(&indices));
        self.n_indices = indices.len() as u32;
    }

    /// Recalculate whether the chunk is flush
    fn update_flush(&mut self) {
        self.flush = 0;
        let mut break_pos = false;
        let mut break_neg = false;
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                if !self.grid[(0,y,z)].is_opaque() {
                    self.flush |= 1 << 0;
                    break_pos = true;
                }
                if! self.grid[(CHUNK_SIZE-1,y,z)].is_opaque() {
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
                if !self.grid[(x,0,z)].is_opaque() {
                    self.flush |= 1 << 2;
                    break_pos = true;
                }
                if !self.grid[(x,CHUNK_SIZE-1,z)].is_opaque() {
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
                if !self.grid[(x,y,0)].is_opaque() {
                    self.flush |= 1 << 4;
                    break_pos = true;
                }
                if !self.grid[(x,y,CHUNK_SIZE-1)].is_opaque() {
                    self.flush |= 1 << 5;
                    break_neg = true;
                }
                if break_neg && break_pos {
                    break;
                }
            }
        }
    }

    /// Update the cubes adjacent to my_coord so that adjacent faces are removed.
    pub fn update_exposed(my_coord: (i32, i32, i32), chunks: &mut FxHashMap<(i32, i32, i32), Self>, graphics: &Graphics) {
        let mut my_exposed = chunks.get(&my_coord).unwrap().exposed;
        let my_flush = chunks.get(&my_coord).unwrap().flush;
        if (my_flush & (1<<0)) == 0 {
            if let Some(c) = chunks.get_mut(&(my_coord.0+1, my_coord.1, my_coord.2)) {
                if (c.flush & (1<<1)) == 0 {
                    my_exposed |= 1 << 0;
                    c.set_exposed(c.exposed | (1<<1), graphics)
                }
            }
        }
        if (my_flush & (1<<1)) == 0 {
            if let Some(c) = chunks.get_mut(&(my_coord.0-1, my_coord.1, my_coord.2)) {
                if (c.flush & (1<<0)) == 0 {
                    my_exposed |= 1 << 1;
                    c.set_exposed(c.exposed | (1<<0), graphics)
                }
            }
        }
        if (my_flush & (1<<2)) == 0 {
            if let Some(c) = chunks.get_mut(&(my_coord.0, my_coord.1+1, my_coord.2)) {
                if (c.flush & (1<<3)) == 0 {
                    my_exposed |= 1 << 2;
                    c.set_exposed(c.exposed | (1<<3), graphics)
                }
            }
        }
        if (my_flush & (1<<3)) == 0 {
            if let Some(c) = chunks.get_mut(&(my_coord.0, my_coord.1-1, my_coord.2)) {
                if (c.flush & (1<<2)) == 0 {
                    my_exposed |= 1 << 3;
                    c.set_exposed(c.exposed | (1<<2), graphics)
                }
            }
        }
        if (my_flush & (1<<4)) == 0 {
            if let Some(c) = chunks.get_mut(&(my_coord.0, my_coord.1, my_coord.2+1)) {
                if (c.flush & (1<<5)) == 0 {
                    my_exposed |= 1 << 4;
                    c.set_exposed(c.exposed | (1<<5), graphics)
                }
            }
        }
        if (my_flush & (1<<5)) == 0 {
            if let Some(c) = chunks.get_mut(&(my_coord.0, my_coord.1, my_coord.2-1)) {
                if (c.flush & (1<<4)) == 0 {
                    my_exposed |= 1 << 5;
                    c.set_exposed(c.exposed | (1<<4), graphics)
                }
            }
        }

        chunks.get_mut(&my_coord).unwrap().set_exposed(my_exposed, graphics);
    }

    pub fn set_detail(&mut self, detail: usize, graphics: &Graphics) {
        if detail == self.detail {return;}
        self.detail = detail;
        self.update_model(graphics); // Update the model with the new detail
    }

    fn set_exposed(&mut self, exposed: u8, graphics: &Graphics) {
        if exposed == self.exposed { return; }
        self.exposed = exposed;
        self.update_model(graphics);
    }

    /// Update the graphics buffer in the grid from the Rigid body.
    pub fn get_uniform(&self, body: &RigidBody, camera: &Camera) -> ModelUniform {
        let pos = (body.pos - body.ori * body.com_pos).cast().unwrap();
        let ori = body.ori.cast().unwrap();
        let model = Matrix4::from_translation(pos + ori * self.global_pos - camera.pos)
            * Matrix4::from(ori);
        ModelUniform::new(model.cast().unwrap())
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        self.buffer.bind(renderer);
        self.vertex_buffer.bind(renderer);
        self.index_buffer.bind(renderer);
        renderer.draw_indices(self.n_indices);
    }

    pub(crate) fn copy_buffer(&self, renderer: &mut Renderer, buffer: &super::StorageBuffer, index:u32) {
        self.buffer.copy_from_storage_buffer(renderer, buffer, index as u64 *std::mem::size_of::<ModelUniform>() as u64);
    }
}

pub struct GridTexture {
    textures: [Texture; 4],
}
impl GridTexture {
    pub fn new(graphics: &Graphics, camera: &Camera, bytes: &[u8]) -> Self {
        let loaded_image = image::load_from_memory(bytes).unwrap();
        let mut rgba = loaded_image.to_rgba8();
        let dimensions = loaded_image.dimensions();
        let t0 = Texture::from_image(graphics, camera, &rgba, dimensions);
        Self::subsample(&mut rgba, dimensions);
        let t1 = Texture::from_image(graphics, camera, &rgba, dimensions);
        Self::subsample(&mut rgba, dimensions);
        let t2 = Texture::from_image(graphics, camera, &rgba, dimensions);
        Self::subsample(&mut rgba, dimensions);
        let t3 = Texture::from_image(graphics, camera, &rgba, dimensions);
        Self {
            textures: [t0, t1, t2, t3]
        }
    }

    fn subsample(pixels: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, dimensions: (u32,u32)) {
        const TEXTURE_WIDTH: u32 = 16;

        for x in (0..dimensions.0).step_by(TEXTURE_WIDTH as usize) {
            for y in (0..dimensions.1).step_by(TEXTURE_WIDTH as usize) {
                let mut out_block = Vec::new();
                for px in (0..TEXTURE_WIDTH).step_by(2) {
                    let mut out_vec = Vec::new();
                    let x = px + x;
                    for py in (0..TEXTURE_WIDTH).step_by(2) {
                        let y = py + y;
                        let mut merged: Rgba<u8> = Rgba([0; 4]);
                        let block = [pixels[(x, y)], pixels[(x+1, y)], pixels[(x+1, y+1)], pixels[(x, y+1)]];
                        for i in 0..4 {
                            merged[i] = (block.map(|p| p[i] as u32).iter().sum::<u32>() / 4) as u8;
                        }
                        out_vec.push(merged);
                    }
                    out_block.push(out_vec);
                }

                // Write the pixels
                for px in 0..(TEXTURE_WIDTH/2) {
                    let x = px + x;
                    for py in 0..(TEXTURE_WIDTH/2) {
                        let y = py + y;
                        pixels[(x,y)] = out_block[px as usize][py as usize];
                        pixels[(x+TEXTURE_WIDTH/2,y)] = out_block[px as usize][py as usize];
                        pixels[(x,y+TEXTURE_WIDTH/2)] = out_block[px as usize][py as usize];
                        pixels[(x+TEXTURE_WIDTH/2,y+TEXTURE_WIDTH/2)] = out_block[px as usize][py as usize];
                    }
                }
            }
        }
    }

    pub fn bind(&self, renderer: &mut Renderer, detail: usize) {
        self.textures[detail-1].bind(renderer);
    }
}

fn add_block(grid: &Grid<GraphicalBlock>, x: u32, y: u32, z: u32, detail_skip: u32, index_offset: &mut u16, vertices: &mut Vec<BlockVertex>, indices: &mut Vec<u16>) {
    let block = grid[(x,y,z)];
    let last = CHUNK_SIZE - detail_skip;
    let x0 = x;
    let x1 = ((x+detail_skip) % 16) | (((x+detail_skip)/16) << 24);
    let y0 = y << 4;
    let y1 = (((y+detail_skip) % 16) << 4) | (((y+detail_skip)/16) << 25);
    let z0 = z << 8;
    let z1 = (((z+detail_skip) % 16) << 8) | (((z+detail_skip)/16) << 26);

    if x == 0 || !grid[(x-detail_skip,y,z)].is_opaque() {
        let face = x0 | (1 << 12);
        let uv = block.get_uv_forward();
        vertices.push(BlockVertex {data: uv.0|y0|z0|face});
        vertices.push(BlockVertex {data: uv.1|y1|z0|face});
        vertices.push(BlockVertex {data: uv.2|y1|z1|face});
        vertices.push(BlockVertex {data: uv.3|y0|z1|face});
        indices.push(0+*index_offset);
        indices.push(2+*index_offset);
        indices.push(1+*index_offset);
        indices.push(0+*index_offset);
        indices.push(3+*index_offset);
        indices.push(2+*index_offset);
        *index_offset += 4;
    }
    if y == 0 || !grid[(x,y-detail_skip,z)].is_opaque() {
        let face = y0 | (3 << 12);
        let uv = block.get_uv_left();
        vertices.push(BlockVertex {data: uv.0|x0|z0|face});
        vertices.push(BlockVertex {data: uv.1|x1|z0|face});
        vertices.push(BlockVertex {data: uv.2|x1|z1|face});
        vertices.push(BlockVertex {data: uv.3|x0|z1|face});
        indices.push(0+*index_offset);
        indices.push(1+*index_offset);
        indices.push(2+*index_offset);
        indices.push(0+*index_offset);
        indices.push(2+*index_offset);
        indices.push(3+*index_offset);
        *index_offset += 4;
    }
    if z == 0 || !grid[(x,y,z-detail_skip)].is_opaque() {
        let face = z0 | (5 << 12);
        let uv = block.get_uv_up();
        vertices.push(BlockVertex {data: uv.0|x0|y0|face});
        vertices.push(BlockVertex {data: uv.1|x1|y0|face});
        vertices.push(BlockVertex {data: uv.2|x1|y1|face});
        vertices.push(BlockVertex {data: uv.3|x0|y1|face});
        indices.push(0+*index_offset);
        indices.push(2+*index_offset);
        indices.push(1+*index_offset);
        indices.push(0+*index_offset);
        indices.push(3+*index_offset);
        indices.push(2+*index_offset);
        *index_offset += 4;
    }
    if x == last || !grid[(x+detail_skip,y,z)].is_opaque() {
        let face = x1; // Normal or constant face
        let uv = block.get_uv_backward();
        vertices.push(BlockVertex {data: uv.0|y0|z0|face});
        vertices.push(BlockVertex {data: uv.1|y1|z0|face});
        vertices.push(BlockVertex {data: uv.2|y1|z1|face});
        vertices.push(BlockVertex {data: uv.3|y0|z1|face});
        indices.push(0+*index_offset);
        indices.push(1+*index_offset);
        indices.push(2+*index_offset);
        indices.push(0+*index_offset);
        indices.push(2+*index_offset);
        indices.push(3+*index_offset);
        *index_offset += 4;
    }
    if y == last || !grid[(x,y+detail_skip,z)].is_opaque() {
        let face = y1 | (2 << 12);
        let uv = block.get_uv_right();
        vertices.push(BlockVertex {data: uv.0|x0|z0|face});
        vertices.push(BlockVertex {data: uv.1|x1|z0|face});
        vertices.push(BlockVertex {data: uv.2|x1|z1|face});
        vertices.push(BlockVertex {data: uv.3|x0|z1|face});
        indices.push(0+*index_offset);
        indices.push(2+*index_offset);
        indices.push(1+*index_offset);
        indices.push(0+*index_offset);
        indices.push(3+*index_offset);
        indices.push(2+*index_offset);
        *index_offset += 4;
    }
    if z == last || !grid[(x,y,z+detail_skip)].is_opaque() {
        let face = z1 | (4 << 12);
        let uv = block.get_uv_down();
        vertices.push(BlockVertex {data: uv.0|x0|y0|face});
        vertices.push(BlockVertex {data: uv.1|x1|y0|face});
        vertices.push(BlockVertex {data: uv.2|x1|y1|face});
        vertices.push(BlockVertex {data: uv.3|x0|y1|face});
        indices.push(0+*index_offset);
        indices.push(1+*index_offset);
        indices.push(2+*index_offset);
        indices.push(0+*index_offset);
        indices.push(2+*index_offset);
        indices.push(3+*index_offset);
        *index_offset += 4;
    }
}

fn add_clutter(grid: &Grid<GraphicalBlock>, x: u32, y: u32, z: u32, detail_skip: u32, index_offset: &mut u16, vertices: &mut Vec<BlockVertex>, indices: &mut Vec<u16>) {
    let block = grid[(x,y,z)];
    let last = CHUNK_SIZE - detail_skip;
    let x0 = x;
    let x1 = ((x+detail_skip) % 16) | (((x+detail_skip)/16) << 24);
    let y0 = y << 4;
    let y1 = (((y+detail_skip) % 16) << 4) | (((y+detail_skip)/16) << 25);
    let z0 = z << 8;
    let z1 = (((z+detail_skip) % 16) << 8) | (((z+detail_skip)/16) << 26);

    let shifted_id = (block.id - 0x100) as u32;
    let uv = (
        ((shifted_id + 1) << 16) | (7 << 20),
        ((shifted_id + 1) << 16) | (6 << 20),
        ((shifted_id + 0) << 16) | (6 << 20),
        ((shifted_id + 0) << 16) | (7 << 20),
    );

    let face = 6 << 12; // Null

    vertices.push(BlockVertex {data: uv.0|x0|y0|z0|face});
    vertices.push(BlockVertex {data: uv.1|x1|y1|z0|face});
    vertices.push(BlockVertex {data: uv.2|x1|y1|z1|face});
    vertices.push(BlockVertex {data: uv.3|x0|y0|z1|face});
    indices.push(0+*index_offset);
    indices.push(1+*index_offset);
    indices.push(2+*index_offset);
    indices.push(0+*index_offset);
    indices.push(2+*index_offset);
    indices.push(3+*index_offset);
    indices.push(0+*index_offset);
    indices.push(2+*index_offset);
    indices.push(1+*index_offset);
    indices.push(0+*index_offset);
    indices.push(3+*index_offset);
    indices.push(2+*index_offset);
    *index_offset += 4;

    vertices.push(BlockVertex {data: uv.0|x1|y0|z0|face});
    vertices.push(BlockVertex {data: uv.1|x0|y1|z0|face});
    vertices.push(BlockVertex {data: uv.2|x0|y1|z1|face});
    vertices.push(BlockVertex {data: uv.3|x1|y0|z1|face});
    indices.push(0+*index_offset);
    indices.push(1+*index_offset);
    indices.push(2+*index_offset);
    indices.push(0+*index_offset);
    indices.push(2+*index_offset);
    indices.push(3+*index_offset);
    indices.push(0+*index_offset);
    indices.push(2+*index_offset);
    indices.push(1+*index_offset);
    indices.push(0+*index_offset);
    indices.push(3+*index_offset);
    indices.push(2+*index_offset);
    *index_offset += 4;
}