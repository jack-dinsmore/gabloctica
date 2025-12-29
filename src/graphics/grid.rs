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

pub const CHUNK_SIZE: u32 = 15;
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
                for z in 0..CHUNK_SIZE {
                    self[(x,y,z)] = ((x+y+z)%2) as u16 + 1;
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

                    let mut vertex_offset = 0;
                    vertex_offset += x;
                    vertex_offset += y << 4;
                    vertex_offset += z << 8;
                    vertex_offset += (typ as u32) << 16;

                    //00_uv_n_zyx
                    if x == CHUNK_SIZE-1 || self[(x+1,y,z)] == 0 {
                        vertices.push(Vertex {data: 0x00_00_0_001 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_01_0_011 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_11_0_111 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_10_0_101 + vertex_offset});
                        indices.push(0+index_offset);
                        indices.push(1+index_offset);
                        indices.push(2+index_offset);
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(3+index_offset);
                        index_offset += 4;
                    }
                    if x == 0 || self[(x-1,y,z)] == 0 {
                        vertices.push(Vertex {data: 0x00_00_1_000 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_01_1_010 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_11_1_110 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_10_1_100 + vertex_offset});
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(1+index_offset);
                        indices.push(0+index_offset);
                        indices.push(3+index_offset);
                        indices.push(2+index_offset);
                        index_offset += 4;
                    }
                    if y == CHUNK_SIZE-1 || self[(x,y+1,z)] == 0 {
                        vertices.push(Vertex {data: 0x00_00_2_010 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_01_2_011 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_11_2_111 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_10_2_110 + vertex_offset});
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(1+index_offset);
                        indices.push(0+index_offset);
                        indices.push(3+index_offset);
                        indices.push(2+index_offset);
                        index_offset += 4;
                    }
                    if y == 0 || self[(x,y-1,z)] == 0 {
                        vertices.push(Vertex {data: 0x00_00_3_000 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_01_3_001 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_11_3_101 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_10_3_100 + vertex_offset});
                        indices.push(0+index_offset);
                        indices.push(1+index_offset);
                        indices.push(2+index_offset);
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(3+index_offset);
                        index_offset += 4;
                    }
                    if z == CHUNK_SIZE-1 || self[(x,y,z+1)] == 0 {
                        vertices.push(Vertex {data: 0x00_00_4_100 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_01_4_101 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_11_4_111 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_10_4_110 + vertex_offset});
                        indices.push(0+index_offset);
                        indices.push(1+index_offset);
                        indices.push(2+index_offset);
                        indices.push(0+index_offset);
                        indices.push(2+index_offset);
                        indices.push(3+index_offset);
                        index_offset += 4;
                    }
                    if z == 0 || self[(x,y,z-1)] == 0 {
                        vertices.push(Vertex {data: 0x00_00_5_000 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_01_5_001 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_11_5_011 + vertex_offset});
                        vertices.push(Vertex {data: 0x00_10_5_010 + vertex_offset});
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