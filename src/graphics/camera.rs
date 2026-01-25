use crate::graphics::{Renderer, ResourceType};
use crate::graphics::{Graphics, resource::UniformBuffer};
use crate::graphics::resource::Uniform;
use cgmath::{EuclideanSpace, Matrix4, Point3, Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}
impl Uniform for CameraUniform {
    const TYPE: ResourceType = ResourceType::Camera;
}
impl CameraUniform {
    fn new(view_proj: Matrix4<f32>) -> Self {
        Self {
            view_proj: (OPENGL_TO_WGPU_MATRIX * view_proj).into()
        }
    }
}



pub struct Camera {
    pub pos: Vector3<f32>,
    pub theta: f32,
    pub phi: f32,
    pub fovy: f32,
    pub up: Vector3<f32>,

    znear: f32,
    zfar: f32,
    aspect: f32,

    buffer: UniformBuffer<CameraUniform>,
}

impl Camera {
    pub fn new(graphics: &Graphics) -> Self {
        let aspect = graphics.surface_config.width as f32 / graphics.surface_config.height as f32;
        let up = Vector3::new(0., 0., 1.);
        let buffer = UniformBuffer::new(graphics);

        Self {
            pos: Vector3::new(0., 0., (5.)*16.),
            theta: 1.57,
            phi: 0.,
            up,
            fovy: 45., // Degrees
            
            znear: 0.1,
            zfar: 1000.,
            aspect,

            buffer,
        }
    }
    
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
    }

    pub fn get_forward(&self) -> Vector3<f32> {
        Vector3::new(
            self.theta.sin() * self.phi.cos(),
            self.theta.sin() * self.phi.sin(),
            self.theta.cos(),
        )
    }

    pub fn get_right(&self) -> Vector3<f32> {
        Vector3::new(
            self.phi.sin(),
            -self.phi.cos(),
            0.,
        )
    }

    pub fn get_up(&self) -> Vector3<f32> {
        Vector3::new(
            -self.theta.cos() * self.phi.cos(),
            -self.theta.cos() * self.phi.sin(),
            self.theta.sin(),
        )
    }

    pub fn update_buffer(&self, graphics: &Graphics) {
        let view = cgmath::Matrix4::look_at_rh(
            Point3::new(0., 0., 0.),
            Point3::from_vec(self.get_forward()),
            self.up
        );
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        let uniform = CameraUniform::new(proj * view);
        self.buffer.write(graphics, uniform);
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        self.buffer.bind(renderer);
    }
}