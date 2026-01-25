use crate::graphics::{Renderer, Vertex};

use super::*;
use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

pub struct UniformBuffer<T: Uniform> {
    pub(in crate::graphics) buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    phantom_data: PhantomData<T>
}

pub struct StorageBuffer {
    pub(in crate::graphics) buffer: wgpu::Buffer,
}

pub struct VertexBuffer<V: Vertex> {
    pub(in crate::graphics) buffer: wgpu::Buffer,
    phantom: PhantomData<V>
}

pub struct IndexBuffer {
    pub(in crate::graphics) buffer: wgpu::Buffer,
}

impl<T: Uniform> UniformBuffer<T> {
    pub fn new(graphics: &Graphics) -> Self {
        let data = [T::zeroed()];
        let descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };
        let buffer = graphics.device.create_buffer_init(&descriptor);

        let layout = &graphics.get_layout(T::TYPE);
        let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout, 
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            buffer,
            bind_group,
            phantom_data: PhantomData {}
        }
    }

    pub fn write(&self, graphics: &Graphics, data: T) {
        graphics.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }

    pub fn copy_from_storage_buffer(&self, renderer: &mut Renderer, storage_buffer: &StorageBuffer, offset: u64) {
        renderer.encoder().copy_buffer_to_buffer(&storage_buffer.buffer, offset, &self.buffer, 0, std::mem::size_of::<T>() as u64);
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        let group = renderer.get_group(T::TYPE);
        renderer.render_pass.as_mut().unwrap().set_bind_group(group, &self.bind_group, &[]);
    }
}

impl StorageBuffer {
    pub fn new(graphics: &Graphics, size: usize) -> Self {
        let descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: &vec![0; size],
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        };
        let buffer = graphics.device.create_buffer_init(&descriptor);

        Self {
            buffer,
        }
    }

    pub fn write<T: NoUninit>(&self, graphics: &Graphics, data: &[T]) {
        graphics.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }

    pub fn write_offset<T: NoUninit>(&self, graphics: &Graphics, data: &[T], offset: u64) {
        graphics.queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data));
    }
}

impl<V: Vertex> VertexBuffer<V> {
    pub fn new(graphics: &Graphics, n_vertices: usize) -> Self {
        let descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &vec![0; n_vertices*std::mem::size_of::<V>()],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        };
        let buffer = graphics.device.create_buffer_init(&descriptor);

        Self {
            buffer,
            phantom: PhantomData {},
        }
    }

    pub fn write<T: NoUninit>(&self, graphics: &Graphics, data: Vec<T>) {
        graphics.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&data));
    }
    
    pub fn bind(&self, renderer: &mut Renderer) {
        let render_pass = renderer.render_pass.as_mut().unwrap();
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
    }
}

impl IndexBuffer {
    pub fn new(graphics: &Graphics, size: usize) -> Self {
        let descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &vec![0; size * std::mem::size_of::<u16>()],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        };
        let buffer = graphics.device.create_buffer_init(&descriptor);

        Self {
            buffer,
        }
    }

    pub fn write<T: NoUninit>(&self, graphics: &Graphics, data: Vec<T>) {
        graphics.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&data));
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        let render_pass = renderer.render_pass.as_mut().unwrap();
        render_pass.set_index_buffer(self.buffer.slice(..), wgpu::IndexFormat::Uint16);
    }
}