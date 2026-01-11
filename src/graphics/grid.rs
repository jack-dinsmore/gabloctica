use std::ops::{Index, IndexMut};
use cgmath::{Matrix4, Quaternion, Vector3};
use image::{GenericImageView, ImageBuffer, Rgba};
use wgpu::util::DeviceExt;

use crate::graphics::{Camera, Graphics, Texture};
use crate::graphics::resource::{UniformBuffer, Uniform};
use crate::graphics::vertex::Vertex;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    model: [[f32; 4]; 4],
}
impl Uniform for ModelUniform {
    const GROUP: u32 = 1;
}
impl ModelUniform {
    fn new(mat: Matrix4<f32>) -> Self {
        Self {
            model: mat.into()
        }
    }
}

pub const CHUNK_SIZE: u32 = 16;
const VERTEX_CAPACITY: usize = 0x10000;

pub struct CubeGrid {
    data: [u16; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize],
    /// Offset of the chunk from the rigid body center (i.e. the center of mass)
    pub global_pos: Vector3<f32>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    n_indices: u32,

    pub buffer: UniformBuffer<ModelUniform>,
    detail: usize,
}

impl CubeGrid {
    pub fn new(graphics: &Graphics) -> Self {
        const INDEX_CAPACITY: usize = (VERTEX_CAPACITY/4) * 6;
        let data = [0; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize];

        let vertex_buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&[Vertex { data: 0}; VERTEX_CAPACITY]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&[0; INDEX_CAPACITY]),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let buffer = UniformBuffer::new(graphics);

        Self {
            data,
            global_pos: Vector3::new(0., 0., 0.),
            vertex_buffer,
            index_buffer,
            n_indices: 0,
            detail: 1,

            buffer,
        }
    }

    pub fn set_data(&mut self, data: [u16; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize]) {
        self.data = data
    }

    pub fn demo(&mut self) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    self[(x,y,z)] = 7;
                }
            }
        }
    }

    // Create vertex and index buffers from the block layout. face_mask should be set to one for hidden faces
    pub fn update_model(&mut self, graphics: &Graphics, detail: usize, face_mask: u8) {
        // Right now I'm just making vertices for every block, including faces that don't face outwards.
        // OPTIMIZE implement face_mask
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u16;
        let detail_skip = 2u32.pow(detail as u32-1);
        let last = CHUNK_SIZE - detail_skip;
        if detail_skip == 0 {
            panic!("{}", detail); // TODO remove
        }
        for x in (0..CHUNK_SIZE).step_by(detail_skip as usize) {
            for y in (0..CHUNK_SIZE).step_by(detail_skip as usize) {
                for z in (0..CHUNK_SIZE).step_by(detail_skip as usize) {
                    let typ = self[(x,y,z)];
                    if typ == 0 {continue;}

                    let x0 = x;
                    let x1 = ((x+detail_skip) % 16) | (((x+detail_skip)/16) << 24);
                    let y0 = y << 4;
                    let y1 = (((y+detail_skip) % 16) << 4) | (((y+detail_skip)/16) << 25);
                    let z0 = z << 8;
                    let z1 = (((z+detail_skip) % 16) << 8) | (((z+detail_skip)/16) << 26);
                    let u0 = (typ as u32 & 0xf) << 16;
                    let u1 = ((typ as u32 & 0xf) + 1) << 16;
                    let v0 = ((typ as u32 >> 4) & 0xf) << 20;
                    let v1 = (((typ as u32 >> 4) & 0xf) + 1) << 20;

                    if x == 0 || self[(x-detail_skip,y,z)] == 0 {
                        let face = x0 | (1 << 12);
                        vertices.push(Vertex {data: u0|v0|y0|z0|face});
                        vertices.push(Vertex {data: u0|v1|y1|z0|face});
                        vertices.push(Vertex {data: u1|v1|y1|z1|face});
                        vertices.push(Vertex {data: u1|v0|y0|z1|face});
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(1+index_offset);
                        indices.push(0+index_offset);
                        indices.push(3+index_offset);
                        indices.push(2+index_offset);
                        index_offset += 4;
                    }
                    if y == 0 || self[(x,y-detail_skip,z)] == 0 {
                        let face = y0 | (3 << 12);
                        vertices.push(Vertex {data: u0|v0|x0|z0|face});
                        vertices.push(Vertex {data: u0|v1|x1|z0|face});
                        vertices.push(Vertex {data: u1|v1|x1|z1|face});
                        vertices.push(Vertex {data: u1|v0|x0|z1|face});
                        indices.push(0+index_offset);
                        indices.push(1+index_offset);
                        indices.push(2+index_offset);
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(3+index_offset);
                        index_offset += 4;
                    }
                    if z == 0 || self[(x,y,z-detail_skip)] == 0 {
                        let face = z0 | (5 << 12);
                        vertices.push(Vertex {data: u0|v0|x0|y0|face});
                        vertices.push(Vertex {data: u0|v1|x1|y0|face});
                        vertices.push(Vertex {data: u1|v1|x1|y1|face});
                        vertices.push(Vertex {data: u1|v0|x0|y1|face});
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(1+index_offset);
                        indices.push(0+index_offset);
                        indices.push(3+index_offset);
                        indices.push(2+index_offset);
                        index_offset += 4;
                    }
                    if x == last || self[(x+detail_skip,y,z)] == 0 {
                        let face = x1; // Normal or constant face
                        vertices.push(Vertex {data: u0|v0|y0|z0|face});
                        vertices.push(Vertex {data: u0|v1|y1|z0|face});
                        vertices.push(Vertex {data: u1|v1|y1|z1|face});
                        vertices.push(Vertex {data: u1|v0|y0|z1|face});
                        indices.push(0+index_offset);
                        indices.push(1+index_offset);
                        indices.push(2+index_offset);
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(3+index_offset);
                        index_offset += 4;
                    }
                    if y == last || self[(x,y+detail_skip,z)] == 0 {
                        let face = y1 | (2 << 12);
                        vertices.push(Vertex {data: u0|v0|x0|z0|face});
                        vertices.push(Vertex {data: u0|v1|x1|z0|face});
                        vertices.push(Vertex {data: u1|v1|x1|z1|face});
                        vertices.push(Vertex {data: u1|v0|x0|z1|face});
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(1+index_offset);
                        indices.push(0+index_offset);
                        indices.push(3+index_offset);
                        indices.push(2+index_offset);
                        index_offset += 4;
                    }
                    if z == last || self[(x,y,z+detail_skip)] == 0 {
                        let face = z1 | (4 << 12);
                        vertices.push(Vertex {data: u0|v0|x0|y0|face});
                        vertices.push(Vertex {data: u0|v1|x1|y0|face});
                        vertices.push(Vertex {data: u1|v1|x1|y1|face});
                        vertices.push(Vertex {data: u1|v0|x0|y1|face});
                        indices.push(0+index_offset);
                        indices.push(1+index_offset);
                        indices.push(2+index_offset);
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(3+index_offset);
                        index_offset += 4;
                    }
                }
            }
        }
        if vertices.len() > VERTEX_CAPACITY {
            panic!("too many vertices");
        }

        graphics.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        graphics.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
        self.n_indices = indices.len() as u32;
        self.detail = detail;
    }

    pub fn get_uniform(&self, pos: Vector3<f32>, ori: Quaternion<f32>, camera: &Camera) -> ModelUniform {
        let model = Matrix4::from_translation(pos + ori * self.global_pos - camera.pos)
            * Matrix4::from(ori);
        ModelUniform::new(model)
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        self.buffer.bind(render_pass);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}

impl Index<(u32, u32, u32)> for CubeGrid {
    type Output = u16;

    fn index(&self, index: (u32, u32, u32)) -> &Self::Output {
        &self.data[(index.0 + index.1*CHUNK_SIZE + index.2*CHUNK_SIZE*CHUNK_SIZE) as usize]
    }
}

impl IndexMut<(u32, u32, u32)> for CubeGrid {
    fn index_mut(&mut self, index: (u32, u32, u32)) -> &mut Self::Output {
        &mut self.data[(index.0 + index.1*CHUNK_SIZE + index.2*CHUNK_SIZE*CHUNK_SIZE) as usize]
    }
}


pub struct GridTexture {
    textures: [Texture; 4],
}
impl GridTexture {
    pub fn new(graphics: &Graphics, bytes: &[u8]) -> Self {
        let loaded_image = image::load_from_memory(bytes).unwrap();
        let mut rgba = loaded_image.to_rgba8();
        let dimensions = loaded_image.dimensions();
        let t0 = Texture::from_image(graphics, &rgba, dimensions);
        Self::subsample(&mut rgba, dimensions);
        let t1 = Texture::from_image(graphics, &rgba, dimensions);
        Self::subsample(&mut rgba, dimensions);
        let t2 = Texture::from_image(graphics, &rgba, dimensions);
        Self::subsample(&mut rgba, dimensions);
        let t3 = Texture::from_image(graphics, &rgba, dimensions);
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

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass, detail: usize) {
        self.textures[detail-1].bind(render_pass);
    }
}