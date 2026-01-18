use std::borrow::Cow;
use crate::graphics::{
    Graphics, Renderer, camera::CameraUniform, grid::ModelUniform, lighting::LightUniform, resource::{TEXTURE_GROUP, Uniform}, vertex::Vertex
};
use wgpu::{BindGroupLayoutDescriptor, Device};


#[derive(Debug)]
pub(super) struct ResourceLayout {
    pub(super) layout: wgpu::BindGroupLayout,
}
impl ResourceLayout {
    pub(super) fn new(device: &Device, i: u32) -> Option<Self> {
        let desc = match i {
            CameraUniform::GROUP => BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
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
            ModelUniform::GROUP => BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
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
            LightUniform::GROUP => BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("light_bind_group_layout"),
            },
            TEXTURE_GROUP => BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            },
            _ => return None,
        };

        Some(Self {
            layout: device.create_bind_group_layout(&desc),
        })
    }
}



pub(super) struct ShaderLayout {
    pub(super) layouts: Vec<ResourceLayout>,
    layout: wgpu::PipelineLayout,
}
impl ShaderLayout {
    pub fn new(device: &Device) -> Self {
        // Create a list of components ordered by binding
        let mut i = 0;
        let mut layouts = Vec::new();
        loop {
            match ResourceLayout::new(device, i) {
                Some(l) => layouts.push(l),
                None => break,
            }
            i += 1;
        }

        let layouts_ref = layouts.iter().map(|l| &l.layout).collect::<Vec<_>>();
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &layouts_ref,
            push_constant_ranges: &[],
        });

        Self {
            layout,
            layouts,
        }
    }
}



pub struct Shader {
    render_pipeline: wgpu::RenderPipeline,
}

impl Shader {
    pub fn new(graphics: &Graphics, source: &'static str) -> Self {
        let shader = graphics.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
        });
    
        let render_pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&graphics.shader_layout.layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: graphics.surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::resource::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
        }
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        renderer.render_pass.as_mut().unwrap().set_pipeline(&self.render_pipeline);
    }
}