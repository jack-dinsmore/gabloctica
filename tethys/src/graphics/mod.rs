mod shader;
mod chunk;
mod vertex;
mod camera;
mod resource;
mod lighting;
mod font;
mod block;
mod post;

pub use chunk::{GraphicalChunk, GridTexture, ModelUniform};
pub use camera::Camera;
pub use lighting::Lighting;
pub use shader::{Shader, RenderTarget};
pub use post::PostRenderer;
pub use vertex::*;
pub use font::Font;
pub use resource::*;
pub use block::GraphicalBlock;

use std::sync::Arc;
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};
use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits, LoadOp, MemoryHints, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor
};

pub const MULTISAMPLE_COUNT: u32 = 4;
pub const RENDER_DISTANCE: f32 = 10000.;
pub const NEAR_DISTANCE: f32 = 0.1;
pub const SHADOW_TEXTURE_PIXELS: u32 = 4096; // How many blocks the shadow texture should span

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
pub const OPENGL_TO_WGPU_MATRIX_INVERSE: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 2.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0,-1.0, 1.0),
);


pub struct Graphics {
    pub window: Arc<Window>,
    _instance: Instance,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    layouts: Vec<Option<wgpu::BindGroupLayout>>,
    
    surface_texture: wgpu::Texture,
    surface_texture_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

fn make_post_textures(device: &Device, dimensions: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView, wgpu::Texture, wgpu::TextureView) {
    // Surface texture
    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some("Surface texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: MULTISAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let surface_texture = device.create_texture(&desc);
    let surface_view = surface_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Depth texture
    let desc = wgpu::TextureDescriptor {
        label: Some("Depth texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: MULTISAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let depth_texture = device.create_texture(&desc);
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    (surface_texture, surface_view, depth_texture, depth_view)
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
        let limits = Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
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

        let (surface_texture, surface_texture_view, depth_texture, depth_texture_view) = make_post_textures(&device, (width, height));
    
        let mut output = Graphics {
            window: window.clone(),
            _instance: instance,
            surface,
            surface_config,
            _adapter: adapter,
            device,
            queue,
            layouts: Vec::new(),
            depth_texture_view,
            surface_texture,
            surface_texture_view,
            depth_texture,
        };

        output.make_layout(ResourceType::Texture);
        
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

        {
            let renderer = Renderer::new(&self, &mut encoder, frame_view);
            render(renderer);
        }
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
            let layout = r.create_bind_group_layout(&self.device);
            self.layouts[r as usize] = Some(layout)
        }
    }
}

pub struct Renderer<'a> {
    encoder_ptr: *mut wgpu::CommandEncoder,
    frame_view: wgpu::TextureView,
    surface_texture_view: &'a wgpu::TextureView,
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
            surface_texture_view: &graphics.surface_texture_view,
        }
    }

    pub fn start_shadow(&mut self, camera: &Camera) {
        self.render_pass.take();
        let encoder_ref = self.encoder();
        self.render_pass = Some(encoder_ref.begin_render_pass(&RenderPassDescriptor {
            label: Some("Shadow render pass"),
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
            label: Some("First render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.surface_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
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
            label: Some("Cleared render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.surface_texture_view,
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

    /// Begin drawing onto the render texture
    pub fn start_post(&mut self) {
        self.render_pass.take();
        let encoder_ref = self.encoder();
        self.render_pass = Some(encoder_ref.begin_render_pass(&RenderPassDescriptor {
            label: Some("Post render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLUE),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        }));
    }

    pub fn start_font(&mut self) {
        self.render_pass.take();
        let encoder_ref = self.encoder();
        self.render_pass = Some(encoder_ref.begin_render_pass(&RenderPassDescriptor {
            label: Some("Font render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        }));
    }

    pub fn stop_post(&mut self) {
        self.render_pass.as_mut().unwrap().draw(0..3, 0..1);
    }

    pub fn draw_indices(&mut self, n_indices: u32) {
        self.render_pass.as_mut().unwrap().draw_indexed(0..n_indices, 0, 0..1);
    }

    fn encoder(&mut self) -> &'a mut wgpu::CommandEncoder {
        if let Some(_) = &self.render_pass {
            panic!("You cannot access the encoder when a render pass is open");
        }
        unsafe {&mut *self.encoder_ptr}
    }
    
    fn get_group(&self, typ: ResourceType) -> u32 {
        self.group_map[typ as usize]
    }
}