use crate::graphics::Graphics;
use crate::graphics::components::{Component, CAMERA_GROUP};
use bytemuck::Zeroable;
use cgmath::{EuclideanSpace, Matrix4, Point3, Vector3};
use wgpu::RenderPass;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
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

    component: Component,
}

impl Camera {
    pub fn new(graphics: &Graphics) -> Self {
        let aspect = graphics.surface_config.width as f32 / graphics.surface_config.height as f32;
        let up = Vector3::new(0., 0., 1.);

        let component = Component::new(graphics, CAMERA_GROUP, &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::zeroed()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pos: Vector3::new(0., 1., 2.),
            theta: 2.5,
            phi: 0.,
            up,
            fovy: 45., // Degrees
            
            znear: 0.1,
            zfar: 100.,
            aspect,

            component,
        }
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
            self.theta.sin() * self.phi.sin(),
            self.theta.sin() * -self.phi.cos(),
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

    pub(super) fn update_component(&self, graphics: &Graphics) {
        let view = cgmath::Matrix4::look_at_rh(
            Point3::new(0., 0., 0.),
            Point3::from_vec(self.get_forward()),
            self.up
        );
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        let uniform = CameraUniform::new(proj * view);

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