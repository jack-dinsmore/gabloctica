use std::ops::{Index, IndexMut};
use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::util::DeviceExt;

use crate::graphics::{Camera, Graphics};
use crate::graphics::resource::{Buffer, Uniform};
use crate::graphics::vertex::Vertex;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ModelUniform {
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
    /// Offset of the chunk in rotation from the rigid body axis (i.e. this quaternion rotates to the principal axes)
    pub global_ori: Quaternion<f32>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    n_indices: u32,

    buffer: Buffer<ModelUniform>,
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

        let buffer = Buffer::new(graphics);

        Self {
            data,
            global_pos: Vector3::new(0., 0., 0.),
            global_ori: Quaternion::new(1., 0., 0., 0.),
            vertex_buffer,
            index_buffer,
            n_indices: 0,

            buffer,
        }
    }

    pub fn demo(&mut self, graphics: &Graphics) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 2..(CHUNK_SIZE-2) {
                    self[(x,y,z)] = 2;
                }
            }
        }
        self.update_model(graphics);
    }

    // Create vertex and index buffers from the block layout
    pub fn update_model(&mut self, graphics: &Graphics) {
        // Right now I'm just making vertices for every block, including faces that don't face outwards.
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u16;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let typ = self[(x,y,z)];
                    if typ == 0 {continue;}

                    let x0 = x;
                    let x1 = ((x+1) % 16) | (((x+1)/16) << 24);
                    let y0 = y << 4;
                    let y1 = (((y+1) % 16) << 4) | (((y+1)/16) << 25);
                    let z0 = z << 8;
                    let z1 = (((z+1) % 16) << 8) | (((z+1)/16) << 26);
                    let u0 = (typ as u32 & 0xf) << 16;
                    let u1 = ((typ as u32 & 0xf) + 1) << 16;
                    let v0 = ((typ as u32 >> 4) & 0xf) << 20;
                    let v1 = (((typ as u32 >> 4) & 0xf) + 1) << 20;

                    if x == CHUNK_SIZE-1 || self[(x+1,y,z)] == 0 {
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
                    if x == 0 || self[(x-1,y,z)] == 0 {
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
                    if y == CHUNK_SIZE-1 || self[(x,y+1,z)] == 0 {
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
                    if y == 0 || self[(x,y-1,z)] == 0 {
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
                    if z == CHUNK_SIZE-1 || self[(x,y,z+1)] == 0 {
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
                    if z == 0 || self[(x,y,z-1)] == 0 {
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
                }
            }
        }
        if vertices.len() > VERTEX_CAPACITY {
            panic!("too many vertices");
        }

        graphics.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        graphics.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
        self.n_indices = indices.len() as u32;
    }

    pub fn update_buffer(&self, graphics: &Graphics, pos: Vector3<f32>, ori: Quaternion<f32>, camera: &Camera) {
        let model = Matrix4::from_translation(pos + ori * self.global_pos - camera.pos)
            * Matrix4::from(ori * self.global_ori);
        let uniform = ModelUniform::new(model);
        self.buffer.write(graphics, uniform);
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