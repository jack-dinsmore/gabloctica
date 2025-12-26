use crate::graphics::Graphics;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, Device, RenderPass
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};


pub(super) const CAMERA_BINDING: u32 = 0;
pub(super) const MODEL_BINDING: u32 = 1;

pub struct ComponentLayout {
    pub(super) binding: u32,
    pub(super) layout: BindGroupLayout,
}
impl ComponentLayout {
    pub(super) fn new(device: &Device, i: u32) -> Option<Self> {
        let desc = match i {
            CAMERA_BINDING => BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: i,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            },
            MODEL_BINDING => BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: i,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("model_bind_group_layout"),
            },
            _ => return None,
        };

        Some(Self {
            layout: device.create_bind_group_layout(&desc),
            binding: i
        })
    }
}

pub struct Component {
    pub(super) binding: u32,
    pub(super) bind_group: BindGroup,
    pub(super) buffer: Buffer,
}

impl Component {
    pub fn new(graphics: &Graphics, binding: u32, buffer_descriptor: &BufferInitDescriptor) -> Self {
        let layout = &graphics.shader_layout.layouts[binding as usize];
        let buffer = graphics.device.create_buffer_init(buffer_descriptor);
        let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: layout.binding,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            binding: layout.binding,
            bind_group,
            buffer,
        }
    }

    pub fn bind(&self, render_pass: &mut RenderPass) {
        render_pass.set_bind_group(self.binding, &self.bind_group, &[]);
    }
}