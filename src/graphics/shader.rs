use std::borrow::Cow;
use crate::graphics::{components::ComponentLayout, vertex::Vertex};
use wgpu::{Device, FragmentState, PipelineLayout, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState
};

pub struct ShaderLayout {
    pub(super) layouts: Vec<ComponentLayout>,
    layout: PipelineLayout,
}
impl ShaderLayout {
    pub fn new(device: &Device) -> Self {
        // Create a list of components ordered by binding
        let mut i = 0;
        let mut layouts = Vec::new();
        loop {
            match ComponentLayout::new(device, i) {
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
    render_pipeline: RenderPipeline,
}

impl Shader {
    pub fn new(source: &'static str, device: &Device, layout: &ShaderLayout, swap_chain_format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(source)),
        });
    
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&layout.layout),
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
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
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

    pub fn bind(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
    }
}