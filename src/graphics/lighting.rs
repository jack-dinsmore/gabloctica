use cgmath::Vector3;
use crate::graphics::{Camera, Graphics, Renderer, ResourceType};
use crate::graphics::resource::{UniformBuffer, Uniform};

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
    const TYPE: ResourceType = ResourceType::Lighting;
}


pub struct Lighting {
    pub pos: Vector3<f32>,
    buffer: UniformBuffer<LightUniform>,
}

impl Lighting {
    pub fn new(graphics: &Graphics) -> Self {
        let pos = Vector3::new(0., 1000., 500.);
        let buffer = UniformBuffer::new(graphics);
        Self {
            pos,
            buffer,
        }
    }

    pub fn update_buffer(&self, graphics: &Graphics, camera: &Camera) {
        let uniform = LightUniform::new(self, camera);
        self.buffer.write(graphics, uniform);
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        self.buffer.bind(renderer);
    }
}