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



const CHUNK_SIZE: usize = 16;

const VERTICES: &[Vertex] = &[
    // n, z, y, x
    Vertex { data: 0x00_01_5_000},
    Vertex { data: 0x00_01_5_001},
    Vertex { data: 0x00_01_5_011},
    Vertex { data: 0x00_01_5_010},
    Vertex { data: 0x00_01_3_000},
    Vertex { data: 0x00_01_3_001},
    Vertex { data: 0x00_01_3_101},
    Vertex { data: 0x00_01_3_100},
    Vertex { data: 0x00_01_1_000},
    Vertex { data: 0x00_01_1_010},
    Vertex { data: 0x00_01_1_110},
    Vertex { data: 0x00_01_1_100},
    Vertex { data: 0x00_01_4_100},
    Vertex { data: 0x00_01_4_101},
    Vertex { data: 0x00_01_4_111},
    Vertex { data: 0x00_01_4_110},
    Vertex { data: 0x00_01_2_010},
    Vertex { data: 0x00_01_2_011},
    Vertex { data: 0x00_01_2_111},
    Vertex { data: 0x00_01_2_110},
    Vertex { data: 0x00_01_0_001},
    Vertex { data: 0x00_01_0_011},
    Vertex { data: 0x00_01_0_111},
    Vertex { data: 0x00_01_0_101},
];

const INDICES: &[u16] = &[
    0, 2, 1, 0, 3, 2,
    4, 5, 6, 4, 6, 7,
    8, 10,9 ,8, 11,10,
    12,13,14,12,14,15,
    16,18,17,16,19,18,
    20,21,22,20,22,23,
];

pub struct Chunk {
    data: [u16; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE],
    pub global_pos: Vector3<f32>,
    pub global_ori: Quaternion<f32>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    n_indices: u32,

    buffer: Buffer<ModelUniform>,
}

impl Chunk {
    pub fn new(graphics: &Graphics) -> Self {
        let data = [0; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE];

        let vertex_buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let buffer = Buffer::new(graphics);

        Self {
            data,
            global_pos: Vector3::new(0., 0., 0.),
            global_ori: Quaternion::new(1., 0., 0., 0.),
            vertex_buffer,
            index_buffer,
            n_indices: INDICES.len() as u32,

            buffer,
        }
    }

    pub fn demo(&mut self) {
        self[(8,8,8)] = 1;
        self.update_buffers();
    }

    pub fn update_buffers(&mut self) {
        
    }

    pub fn update_buffer(&self, graphics: &Graphics, camera: &Camera) {
        let model = Matrix4::from_translation(self.global_pos - camera.pos) * Matrix4::from(self.global_ori);
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

impl Index<(usize, usize, usize)> for Chunk {
    type Output = u16;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.data[index.0 + index.1*CHUNK_SIZE + index.2*CHUNK_SIZE*CHUNK_SIZE]
    }
}

impl IndexMut<(usize, usize, usize)> for Chunk {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0 + index.1*CHUNK_SIZE + index.2*CHUNK_SIZE*CHUNK_SIZE]
    }
}