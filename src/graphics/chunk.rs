use std::ops::{Index, IndexMut};

use cgmath::{Quaternion, Vector3};
use wgpu::{RenderPass, Buffer};
use wgpu::util::DeviceExt;

use crate::graphics::Graphics;
use crate::graphics::vertex::Vertex;

const CHUNK_SIZE: usize = 16;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

pub struct Chunk {
    data: [u16; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE],
    pub global_pos: Vector3<f32>,
    pub global_ori: Quaternion<f32>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
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

        Self {
            data,
            global_pos: Vector3::new(0., 0., 0.),
            global_ori: Quaternion::new(1., 0., 0., 0.),
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn demo(&mut self) {
        self[(8,8,8)] = 1;
        self.update_buffers();
    }

    pub fn update_buffers(&mut self) {

    }

    pub(super) fn draw(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
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