pub struct Shader {
    render_pipeline: RenderPipeline
}
use std::borrow::Cow;
use wgpu::{
    Device, FragmentState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState,
};

use crate::graphics::vertex::Vertex;

impl Shader {
    pub fn new(source: &'static str, device: &Device, swap_chain_format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(source)),
        });
    
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(swap_chain_format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
        }
    }

    pub fn get_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }
}