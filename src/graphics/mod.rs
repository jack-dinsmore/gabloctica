mod shader;
mod grid;
mod vertex;
mod camera;
mod resource;
mod lighting;
mod font;

pub use grid::{CubeGrid, GridTexture, CHUNK_SIZE, ModelUniform};
pub use camera::Camera;
pub use lighting::Lighting;
pub use shader::Shader;
pub use resource::{Texture, StorageBuffer, UniformBuffer};

use std::sync::Arc;
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};
use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits, LoadOp, MemoryHints, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor
};

use crate::graphics::shader::ShaderLayout;

pub struct Graphics {
    pub window: Arc<Window>,
    _instance: Instance,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    depth_texture_view: wgpu::TextureView,
    shader_layout: ShaderLayout,
}

impl Graphics {
    pub async fn create(window: Window, proxy: EventLoopProxy<Graphics>) {
        let window = Arc::new(window);
        let instance = Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await.expect("Could not get an adapter (GPU).");
    
        // Get device
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                memory_hints: MemoryHints::Performance,
                trace: Default::default(),
            },
        ).await.expect("Failed to get device");
    
        // Window size
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &surface_config);

        // ShaderLayout
        let shader_layout = ShaderLayout::new(&device);
        let depth_texture_view = Texture::depth_view(&device, width, height);
    
        let output = Graphics {
            window: window.clone(),
            _instance: instance,
            surface,
            surface_config,
            _adapter: adapter,
            device,
            queue,
            shader_layout,
            depth_texture_view,
        };
    
        let _ = proxy.send_event(output);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        // TODO resize the depth texture
    }

    pub fn draw(&self, render: impl FnOnce(Renderer)) {
        let frame = self.surface.get_current_texture().expect("Failed to acquire next swap chain texture.");
        let frame_view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        let renderer = Renderer::new(&self, &mut encoder, frame_view);
        render(renderer);

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        self.window.request_redraw();
    }
}

pub struct Renderer<'a> {
    encoder_ptr: *mut wgpu::CommandEncoder,
    frame_view: wgpu::TextureView,
    render_pass: Option<wgpu::RenderPass<'a>>,
    depth_texture_view: &'a wgpu::TextureView,
}
impl<'a> Renderer<'a> {
    pub fn new(graphics: &'a Graphics, encoder: &'a mut wgpu::CommandEncoder, frame_view: wgpu::TextureView) -> Self {
        let encoder_ptr = encoder as *mut _;

        Self {
            render_pass: None,
            frame_view,
            encoder_ptr,
            depth_texture_view: &graphics.depth_texture_view,
        }
    }

    pub fn start(&mut self) {
        self.render_pass.take();
        let encoder_ref = self.encoder();
        self.render_pass = Some(encoder_ref.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::GREEN),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        }));
    }

    pub fn clear(&mut self) {
        self.render_pass.take();
        let encoder_ref = self.encoder();
        self.render_pass = Some(encoder_ref.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        }));
    }

    fn encoder(&mut self) -> &'a mut wgpu::CommandEncoder {
        if let Some(_) = &self.render_pass {
            panic!("You cannot access the encoder when a renderpass is open");
        }
        unsafe {&mut *self.encoder_ptr}
    }
}