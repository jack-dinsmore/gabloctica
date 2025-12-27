use crate::graphics::{Camera, Chunk, Graphics, Lighting};
use rustc_hash::FxHashSet;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey},
};


struct KeyState {
    down_set: FxHashSet<KeyCode>
}
impl KeyState {
    pub fn new() -> Self {
        Self {
            down_set: FxHashSet::default(),
        }
    }

    pub fn update(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(code) = event.physical_key {
            if event.state.is_pressed() {
                self.down_set.insert(code);
            } else {
                self.down_set.remove(&code);
            }
        }
    }

    pub fn get(&self, key: KeyCode) -> bool {
        self.down_set.contains(&key)
    }
}


pub struct Game {
    graphics: Graphics,
    key_state: KeyState,
    camera: Camera,
    lighting: Lighting,
    chunks: Vec<Chunk>,
    mouse_motion: (f32, f32)
}

impl Game {
    pub fn new(graphics: Graphics) -> Self {
        let key_state = KeyState::new();
        let chunks = vec![Chunk::new(&graphics)];
        let camera = Camera::new(&graphics);
        let lighting = Lighting::new(&graphics);

        // Set cursor to center of screen
        let size = graphics.window.inner_size();
        let center = PhysicalPosition::new(
            size.width as f64 / 2.0,
            size.height as f64 / 2.0,
        );
        graphics.window.set_cursor_position(center).unwrap();
        
        Self {
            graphics,
            key_state,
            camera,
            chunks,
            mouse_motion: (0., 0.),
            lighting,
        }
    }

    pub fn mouse_moved(&mut self, difference: (f32, f32)) {
        self.mouse_motion = difference;
    }

    pub fn update(&mut self, delta_t: f64) {
        {
            // Move camera pos
            const SPEED: f64 = 2.;
            let forward = self.camera.get_forward();
            let up = self.camera.get_up();
            let right = self.camera.get_right();
            if self.key_state.get(KeyCode::KeyW) {
                self.camera.pos += forward * (SPEED*delta_t) as f32;
            }
            if self.key_state.get(KeyCode::KeyS) {
                self.camera.pos -= forward * (SPEED*delta_t) as f32;
            }
            if self.key_state.get(KeyCode::KeyD) {
                self.camera.pos += right * (SPEED*delta_t) as f32;
            }
            if self.key_state.get(KeyCode::KeyA){
                self.camera.pos -= right * (SPEED*delta_t) as f32;
            }
            if self.key_state.get(KeyCode::KeyQ) {
                self.camera.pos += up * (SPEED*delta_t) as f32;
            }
            if self.key_state.get(KeyCode::KeyE){
                self.camera.pos -= up * (SPEED*delta_t) as f32;
            }
        }

        {
            // Move camera look
            const SPEED: f64 = 0.3;
            self.camera.theta += (SPEED*delta_t) as f32*self.mouse_motion.1;
            self.camera.phi -= (SPEED*delta_t) as f32*self.mouse_motion.0;
            self.mouse_motion = (0., 0.);
            self.camera.theta = self.camera.theta.clamp(0.0001, 3.1415);
        }
    }

    pub fn draw(&mut self) {
        self.graphics.draw(&self.chunks, &self.camera, &self.lighting);
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        self.camera.resize(size); // TODO
        self.graphics.resize(size);
    }
    
    /// Run in response to an event. Return true if you want the window to close.
    pub fn window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::CloseRequested => return true,
            WindowEvent::KeyboardInput { event, .. } => {
                self.key_state.update(&event);
                if let PhysicalKey::Code(code) = &event.physical_key {
                    match code {
                        KeyCode::Escape => return true,
                        _ => ()
                    };
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let size = self.graphics.window.inner_size();
                let center = PhysicalPosition::new(
                    size.width as f64 / 2.0,
                    size.height as f64 / 2.0,
                );
                let difference = ((position.x - center.x) as f32, (position.y - center.y) as f32);
                self.graphics.window.set_cursor_position(center).unwrap();

                self.mouse_moved(difference)
            }
            _ => ()
        };
        false
    }
}