use std::borrow::Cow;
use wgpu::BlendState;

use crate::graphics::{
    Graphics, MULTISAMPLE_COUNT, Renderer, resource::ResourceType, vertex::Vertex
};

pub struct Shader {
    render_pipeline: wgpu::RenderPipeline,
    layout: wgpu::PipelineLayout,
    resources: Vec<ResourceType>,
    render_target: RenderTarget,
}

#[derive(Clone, Copy, Debug)]
pub enum RenderTarget {
    Texture,
    Shadow,
    Screen,
}

impl Shader {
    pub fn new<V: Vertex>(graphics: &mut Graphics, source: &'static str, resources: Vec<ResourceType>, render_target: RenderTarget) -> Self {
        let shader = graphics.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
        });

        // Create shader layout
        for r in &resources {
            graphics.make_layout(*r);
        }
        let layouts = resources.iter().map(|r| graphics.get_layout(*r)).collect::<Vec<_>>();

        let layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        });

        let fragment = if source.contains("@fragment") {
            Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: graphics.surface_config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            })
        } else {
            None
        };

        let depth_stencil = match render_target {
            RenderTarget::Shadow | RenderTarget::Texture => Some(wgpu::DepthStencilState {
                format: super::resource::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            RenderTarget::Screen => None,
        };

        let multisample = match render_target {
            RenderTarget::Shadow | RenderTarget::Screen => Default::default(),
            RenderTarget::Texture => wgpu::MultisampleState {
                count: MULTISAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        // Create shader    
        let buffers: &[wgpu::VertexBufferLayout<'static>] = match V::desc() {
            Some(b) => &[b],
            None => &[],
        };
        let render_pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render pipeline {:?}", render_target)),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers,
                compilation_options: Default::default(),
            },
            fragment,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil,
            multisample,
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            layout,
            resources,
            render_target,
        }
    }

    pub fn bind(&self, renderer: &mut Renderer) {
        renderer.render_pass.as_mut().unwrap().set_pipeline(&self.render_pipeline);
        for (i, resource) in self.resources.iter().enumerate() {
            renderer.group_map[*resource as usize] = i as u32;
        }
    }
}