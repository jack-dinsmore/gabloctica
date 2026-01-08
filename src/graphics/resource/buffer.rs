use super::*;
use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

pub struct UniformBuffer<T: Uniform> {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    phantom_data: PhantomData<T>
}

pub struct StorageBuffer {
    buffer: wgpu::Buffer,
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

        let layout = &graphics.shader_layout.layouts[T::GROUP as usize];
        let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout.layout, 
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

    pub fn copy_from_storage_buffer(&self, encoder: &mut wgpu::CommandEncoder, storage_buffer: &StorageBuffer, offset: u64) {
        encoder.copy_buffer_to_buffer(&storage_buffer.buffer, offset, &self.buffer, 0, std::mem::size_of::<T>() as u64);
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(T::GROUP, &self.bind_group, &[]);
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

    pub fn write<T: NoUninit>(&self, graphics: &Graphics, data: Vec<T>) {
        graphics.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&data));
    }
}