use cgmath::Vector3;
use bytemuck::Zeroable;
use wgpu::RenderPass;
use crate::graphics::{Camera, Graphics, components::{Component, LIGHT_GROUP}};




#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    pos: [f32; 4],
}
impl LightUniform {
    fn new(lighting: &Lighting, camera: &Camera) -> Self {
        let shifted_pos = lighting.pos - camera.pos;
        Self {
            pos: [shifted_pos.x, shifted_pos.y, shifted_pos.z, 0.],
        }
    }
}


pub struct Lighting {
    pos: Vector3<f32>,

    component: Component,
}

impl Lighting {
    pub fn new(graphics: &Graphics) -> Self {
        let pos = Vector3::new(0., 0., 10.);

        let component = Component::new(graphics, LIGHT_GROUP, &wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[LightUniform::zeroed()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pos,
            component,
        }
    }

    pub(super) fn update_component(&self, graphics: &Graphics, camera: &Camera) {
        let uniform = LightUniform::new(self, camera);

        graphics.queue.write_buffer(
            &self.component.buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );
    }

    pub(super) fn bind(&self, render_pass: &mut RenderPass) {
        self.component.bind(render_pass);
    }
}