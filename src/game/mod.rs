use crate::graphics::{Chunk, Graphics};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
};

pub struct Game {
    graphics: Graphics,
    chunks: Vec<Chunk>,
}

impl Game {
    pub fn new(graphics: Graphics) -> Self {
        let chunks = vec![Chunk::new(&graphics)];
        Self {
            graphics,
            chunks,
        }
    }

    fn draw(&mut self) {
        self.graphics.draw(&self.chunks);
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        self.graphics.resize(size);
    }
    
    /// Run in response to an event. Return true if you want the window to close.
    pub fn window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::CloseRequested => return true,
            _ => ()
        };
        false
    }
}