pub mod object;
pub mod planet;

use crate::{game::object::Object, graphics::{Camera, Graphics, GridTexture, Lighting, Shader}, physics::{Collider, CollisionReport, Physics, RigidBody, RigidBodyInit}};
use planet::{Planet, PlanetInit};
use cgmath::Rotation;
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
    physics: Box<Physics>,
    player: RigidBody,
    shader: Shader,
    key_state: KeyState,
    camera: Camera,
    lighting: Lighting,
    texture: GridTexture,
    objects: Vec<Object>,
    mouse_motion: (f32, f32)
}

impl Game {
    pub fn new(graphics: Graphics) -> Self {
        let mut physics = Box::new(Physics::new());
        let key_state = KeyState::new();
        let shader = Shader::new(&graphics, include_str!("../shaders/shader.wgsl"));
        let camera = Camera::new(&graphics);
        let planet = Planet::new(PlanetInit::default());
        let objects = vec![
            Object::new(&graphics, &mut physics, planet.loader(), camera.pos.cast().unwrap())
        ];
        let player = RigidBody::new(&mut physics, RigidBodyInit::default());
        let lighting = Lighting::new(&graphics);

        // let font = Font::new(include_bytes!("/System/Library/Fonts/Supplemental/Rockwell.ttc"));

        // Set cursor to center of screen
        let size = graphics.window.inner_size();
        let center = PhysicalPosition::new(
            size.width as f64 / 2.0,
            size.height as f64 / 2.0,
        );
        graphics.window.set_cursor_position(center).unwrap();

        // Load block texture
        let texture = GridTexture::new(&graphics, include_bytes!("../../assets/texture.png"));
        
        Self {
            graphics,
            key_state,
            camera,
            objects,
            mouse_motion: (0., 0.),
            lighting,
            texture,
            shader,
            physics,
            player,
        }
    }

    pub fn mouse_moved(&mut self, difference: (f32, f32)) {
        self.mouse_motion = difference;
    }
    
    fn mouse_clicked(&mut self, button: winit::event::MouseButton) {
        match button {
            winit::event::MouseButton::Left => {
                // Replace the player's collider with its look ray temporarily
                const LOOK_DIST: f64 = 5.;
                let body_collider = self.player.collider.take();
                let forward = self.camera.get_forward().cast().unwrap();
                self.player.collider = Some(Collider::new_ray(self.camera.pos.cast().unwrap(), forward*LOOK_DIST));

                let mut report = CollisionReport::None;
                let mut collided_object = None;

                for object in &mut self.objects {
                    // The collision function should always pick some over None, but choose the one with the smallest distance to the target otherwise.
                    let new_report = Collider::check_collision(&self.player, &object.body);

                    if new_report > report {
                        report = new_report;
                        collided_object = Some(object);
                    }
                }

                if let Some(o) = collided_object {
                    let place_pos = match report {
                        CollisionReport::Some { p2, .. } => {
                            let offset = o.body.ori.invert() * forward;
                            p2 - offset*0.001
                        }
                        CollisionReport::None => unreachable!(),
                    };
                    o.insert_block(&self.graphics, 1, place_pos);
                }

                // Put the collider back
                self.player.collider = body_collider;
            },
            _ => (),
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        for object in &mut self.objects {
            object.update(&self.graphics, self.camera.pos.cast().unwrap());
        }

        {
            // Move camera pos
            const SPEED: f64 = 200.;
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
            const SPEED: f64 = 0.2;
            self.camera.theta += (SPEED*delta_t) as f32*self.mouse_motion.1;
            self.camera.phi -= (SPEED*delta_t) as f32*self.mouse_motion.0;
            self.mouse_motion = (0., 0.);
            self.camera.theta = self.camera.theta.clamp(0.0001, 3.1415);
        }

        self.physics.update(delta_t);
    }

    pub fn draw(&mut self) {
        // OPTIMIZE avoid all calls of queue.write_buffer.
        self.camera.update_buffer(&self.graphics);
        self.lighting.update_buffer(&self.graphics, &self.camera);
        for object in &mut self.objects {
            object.update_buffer(&self.graphics, &self.camera)
        }
        
        self.graphics.draw(
            |encoder| {
                for object in &self.objects {
                    object.copy_buffers(encoder);
                }
            },

            |render_pass| {
                self.shader.bind(render_pass);
                self.camera.bind(render_pass);
                self.lighting.bind(render_pass);
                for object in &self.objects {
                    object.draw(render_pass, &self.texture)
                }
            }
        );
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        self.camera.resize(size);
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
            },
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    winit::event::ElementState::Pressed => self.mouse_clicked(button),
                    winit::event::ElementState::Released => (),
                }
            }
            _ => ()
        };
        false
    }
}