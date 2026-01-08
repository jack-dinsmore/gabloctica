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

    pub fn draw(&mut self, buffer_pipeline: impl FnOnce(&mut wgpu::CommandEncoder), render_pipeline: impl FnOnce(&mut wgpu::RenderPass)) {
        let frame = self.surface.get_current_texture() .expect("Failed to acquire next swap chain texture.");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        buffer_pipeline(&mut encoder);
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
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
            });
            
            render_pipeline(&mut render_pass);
        } 

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        self.window.request_redraw();
    }
}
