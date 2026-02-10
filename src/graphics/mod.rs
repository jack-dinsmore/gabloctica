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
pub use vertex::*;
pub use font::Font;
pub use resource::*;

use std::sync::Arc;
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};
use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits, LoadOp, MemoryHints, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor
};

pub struct Graphics {
    pub window: Arc<Window>,
    _instance: Instance,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    depth_texture_view: wgpu::TextureView,
    layouts: Vec<Option<wgpu::BindGroupLayout>>,
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
        let mut limits = Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        limits.max_bind_groups = 5;
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: limits,
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

        let depth_texture_view = Texture::depth(&device, (width, height)).1;
    
        let output = Graphics {
            window: window.clone(),
            _instance: instance,
            surface,
            surface_config,
            _adapter: adapter,
            device,
            queue,
            depth_texture_view,
            layouts: Vec::new(),
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
    
    fn get_layout(&self, typ: ResourceType) -> &wgpu::BindGroupLayout {
        self.layouts[typ as usize].as_ref().unwrap()
    }
    
    fn make_layout(&mut self, r: ResourceType) {
        while self.layouts.len() <= r as usize {
            self.layouts.push(None);
        }
        if let None = self.layouts[r as usize] {
            let layout = self.device.create_bind_group_layout(&r.get_descriptor());
            self.layouts[r as usize] = Some(layout)
        }
    }
}

pub struct Renderer<'a> {
    encoder_ptr: *mut wgpu::CommandEncoder,
    frame_view: wgpu::TextureView,
    render_pass: Option<wgpu::RenderPass<'a>>,
    depth_texture_view: &'a wgpu::TextureView,
    group_map: [u32; 5],
}
impl<'a> Renderer<'a> {
    pub fn new(graphics: &'a Graphics, encoder: &'a mut wgpu::CommandEncoder, frame_view: wgpu::TextureView) -> Self {
        let encoder_ptr = encoder as *mut _;

        Self {
            render_pass: None,
            frame_view,
            encoder_ptr,
            depth_texture_view: &graphics.depth_texture_view,
            group_map: [0; 5],
        }
    }

    pub fn start_shadow(&mut self, camera: &Camera) {
        self.render_pass.take();
        let encoder_ref = self.encoder();
        self.render_pass = Some(encoder_ref.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &camera.shadow_texture.view,
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
    
    pub fn draw_indices(&mut self, n_indices: u32) {
        self.render_pass.as_mut().unwrap().draw_indexed(0..n_indices, 0, 0..1);
    }

    fn encoder(&mut self) -> &'a mut wgpu::CommandEncoder {
        if let Some(_) = &self.render_pass {
            panic!("You cannot access the encoder when a renderpass is open");
        }
        unsafe {&mut *self.encoder_ptr}
    }
    
    fn get_group(&self, typ: ResourceType) -> u32 {
        self.group_map[typ as usize]
    }
}