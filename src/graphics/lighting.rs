use cgmath::Vector3;
use bytemuck::Zeroable;
use wgpu::RenderPass;
use crate::graphics::{Camera, Graphics, components::{Component, LIGHT_GROUP}};




#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    pos: [f32; 4],
    normal: [f32; 4],
}
impl LightUniform {
    fn new(lighting: &Lighting, normal: Vector3<f32>) -> Self {
        Self {
            pos: [lighting.pos.x, lighting.pos.y, lighting.pos.z, 0.],
            normal: [normal.x, normal.y, normal.z, 0.],
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

    pub(super) fn update_component(&self, graphics: &Graphics, _camera: &Camera) {
        let uniform = LightUniform::new(self, Vector3::new(1., 0., 0.));

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