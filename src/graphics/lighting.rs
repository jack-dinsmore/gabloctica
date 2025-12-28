use cgmath::Vector3;
use wgpu::RenderPass;
use crate::graphics::{Camera, Graphics};
use crate::graphics::resource::{Buffer, Uniform};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct LightUniform {
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
impl Uniform for LightUniform {
    const GROUP: u32 = 2;
}


pub struct Lighting {
    pos: Vector3<f32>,
    buffer: Buffer<LightUniform>,
}

impl Lighting {
    pub fn new(graphics: &Graphics) -> Self {
        let pos = Vector3::new(-5., -10., 10.);
        let buffer = Buffer::new(graphics);
        Self {
            pos,
            buffer,
        }
    }

    pub fn update_buffer(&self, graphics: &Graphics, camera: &Camera) {
        let uniform = LightUniform::new(self, camera);
        self.buffer.write(graphics, uniform);
    }

    pub fn bind(&self, render_pass: &mut RenderPass) {
        self.buffer.bind(render_pass);
    }
}