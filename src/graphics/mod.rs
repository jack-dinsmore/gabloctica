mod shader;
mod chunk;
mod vertex;

pub use chunk::Chunk;
use shader::Shader;

use std::sync::Arc;
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};
use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
    Instance, Limits, LoadOp, MemoryHints, Operations, PowerPreference, Queue,
    RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, TextureViewDescriptor,
};

pub struct Graphics {
    window: Arc<Window>,
    instance: Instance,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    shader: Shader,
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
    
        // Shaders
        let shader = Shader::new(include_str!("shaders/shader.wgsl"), &device, surface_config.format);
    
        let output = Graphics {
            window: window.clone(),
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            shader,
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
    }

    pub fn draw(&mut self, chunks: &Vec<Chunk>) {
        let frame = self.surface.get_current_texture() .expect("Failed to acquire next swap chain texture.");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::GREEN),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r_pass.set_pipeline(self.shader.get_pipeline());
            for chunk in chunks {
                chunk.draw(&mut r_pass)
            }
        } 

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
