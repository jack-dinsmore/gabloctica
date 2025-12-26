use std::ops::{Index, IndexMut};

use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use wgpu::{RenderPass, Buffer};
use wgpu::util::DeviceExt;

use crate::graphics::Graphics;
use crate::graphics::components::{Component, MODEL_BINDING};
use crate::graphics::vertex::Vertex;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
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
    Vertex { position: [0., 0., 0.], },
    Vertex { position: [0., 1., 0.], },
    Vertex { position: [1., 1., 0.], },
    Vertex { position: [1., 0., 0.], },
    Vertex { position: [0., 0., 1.], },
    Vertex { position: [0., 1., 1.], },
    Vertex { position: [1., 1., 1.], },
    Vertex { position: [1., 0., 1.], },
];

const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3,
    4, 6, 5, 4, 7, 6,
    2, 3, 6, 2, 6, 7,
    0, 1, 4, 0, 4, 5,
    1, 2, 5, 1, 5, 6,
    0, 3, 4, 0, 4, 7,
];

pub struct Chunk {
    data: [u16; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE],
    pub global_pos: Vector3<f32>,
    pub global_ori: Quaternion<f32>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    n_indices: u32,

    component: Component,
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

        let component = Component::new(graphics, MODEL_BINDING, &wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&[ModelUniform::new(Matrix4::identity())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            data,
            global_pos: Vector3::new(0., 0., 0.),
            global_ori: Quaternion::new(1., 0., 0., 0.),
            vertex_buffer,
            index_buffer,
            n_indices: INDICES.len() as u32,

            component
        }
    }

    pub fn demo(&mut self) {
        self[(8,8,8)] = 1;
        self.update_buffers();
    }

    pub fn update_buffers(&mut self) {
        
    }

    pub(super) fn update_component(&self, graphics: &Graphics) {
        let model = Matrix4::from_translation(self.global_pos) * Matrix4::from(self.global_ori);
        let uniform = ModelUniform::new(model);

        graphics.queue.write_buffer(
            &self.component.buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );
    }

    pub(super) fn draw(&self, render_pass: &mut RenderPass) {
        self.component.bind(render_pass);
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