
use crate::graphics::{Graphics, Renderer, ResourceType, Uniform, UniformBuffer};

pub struct PostRenderer {
    bind_group: wgpu::BindGroup,
}
impl PostRenderer {
    pub fn new<T: Uniform>(graphics: &mut Graphics, buffer: &UniformBuffer<T>) -> Self {
        let layout = &graphics.get_layout(ResourceType::Post);
        let bind_group = graphics.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&graphics.surface_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&graphics.depth_texture_view),
                    },
                ],
                label: Some("Texture bind group"),
            }
        );

        Self {
            bind_group,
        }
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        renderer.render_pass.as_mut().unwrap().set_bind_group(0, &self.bind_group, &[]);
    }
}