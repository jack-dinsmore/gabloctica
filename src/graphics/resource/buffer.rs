use super::*;
use wgpu::util::DeviceExt;

pub struct Buffer<T: Uniform> {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    phantom_data: PhantomData<T>
}

impl<T: Uniform> Buffer<T> {
    pub fn new(graphics: &Graphics) -> Self {
        let data = [T::zeroed()];
        let descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Buffer"),
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

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(T::GROUP, &self.bind_group, &[]);
    }
}