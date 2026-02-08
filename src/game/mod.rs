pub mod object;
pub mod planet;

use crate::game::object::ObjectLoader;
use crate::game::object::loader::ShipLoader;
use crate::graphics::*;
use crate::physics::*;
use cgmath::InnerSpace;
use cgmath::Vector3;
use object::Object;
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
    block_shader: Shader,
    flat_shader: Shader,
    key_state: KeyState,
    camera: Camera,
    lighting: Lighting,
    texture: GridTexture,
    objects: Vec<Object>,
    mouse_motion: (f32, f32),
    font: Font,
    fps_counter: FpsCounter,
}

impl Game {
    pub fn new(mut graphics: Graphics) -> Self {
        let mut physics = Box::new(Physics::new());
        let key_state = KeyState::new();
        let block_shader = Shader::new::<BlockVertex>(&mut graphics, include_str!("../shaders/block.wgsl"), vec![
            ResourceType::Camera,
            ResourceType::Model,
            ResourceType::Lighting,
            ResourceType::Texture,
        ]);
        let flat_shader = Shader::new::<FlatVertex>(&mut graphics, include_str!("../shaders/flat.wgsl"), vec![
            ResourceType::Camera,
            ResourceType::Model,
            ResourceType::Texture,
        ]);
        let camera = Camera::new(&graphics);
        let planet = Planet::new(PlanetInit::default());
        let objects = vec![
            // Object::new(&graphics, &mut physics, planet.loader(), RigidBodyInit::default()),
            Object::new(&graphics, &mut physics, ObjectLoader::OneShot(ShipLoader{}), RigidBodyInit { pos:  Vector3::new(15., -9., 50.), ang_vel: Vector3::new(0., 0., -1.), ..Default::default()}),
            Object::new(&graphics, &mut physics, ObjectLoader::OneShot(ShipLoader{}), RigidBodyInit { pos:  Vector3::new(15., 9., 50.), ..Default::default()}),
        ];
        let player = RigidBody::new(&mut physics, RigidBodyInit {pos: Vector3::new(0., 0., 50.), ..Default::default()});
        let lighting = Lighting::new(&graphics);

        // Set cursor to center of screen
        let size = graphics.window.inner_size();
        let center = PhysicalPosition::new(
            size.width as f64 / 2.0,
            size.height as f64 / 2.0,
        );
        graphics.window.set_cursor_position(center).unwrap();

        // Load block texture
        let texture = GridTexture::new(&graphics, include_bytes!("../../assets/texture.png"));

        let font = Font::new(&mut graphics, include_bytes!("../../assets/Rockwell.ttc"));
        
        Self {
            graphics,
            key_state,
            camera,
            objects,
            mouse_motion: (0., 0.),
            lighting,
            texture,
            block_shader,
            flat_shader,
            physics,
            player,
            font,
            fps_counter: FpsCounter::new(),
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

    fn update_gravity(&mut self) {
        let mut gravitating = Vec::new();
        const GRAVITATING_THRESHOLD: f64 = 10.;
        for object in &self.objects {
            if object.body.mass > GRAVITATING_THRESHOLD {
                gravitating.push((object.body.pos, object.body.mass));
            }
        }
        for object in &mut self.objects {
            for (pos, mass) in &gravitating {
                let delta = object.body.pos - pos;
                let grav = -delta * NEWTON_G * *mass * object.body.mass / delta.magnitude().powi(3);
                object.body.add_force(grav);
            }
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        for object in &mut self.objects {
            object.update(&self.graphics, self.camera.pos.cast().unwrap());
        }
        self.fps_counter.update(delta_t);

        // self.update_gravity();

        {
            // Move camera pos
            const SPEED: f64 = 5000.;
            let forward: Vector3<f64> = self.camera.get_forward().cast().unwrap();
            let up: Vector3<f64> = self.camera.get_up().cast().unwrap();
            let right: Vector3<f64> = self.camera.get_right().cast().unwrap();
            if self.key_state.get(KeyCode::KeyW) {
                self.player.add_force(forward * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyS) {
                self.player.add_force(-forward * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyD) {
                self.player.add_force(right * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyA){
                self.player.add_force(-right * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyQ) {
                self.player.add_force(up * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyE){
                self.player.add_force(-up * (SPEED*delta_t));
            }
        }

        {
            // Move camera look
            const SPEED: f64 = 0.2;
            self.camera.pos = self.player.pos.cast().unwrap();
            self.camera.theta += (SPEED*delta_t) as f32 *self.mouse_motion.1;
            self.camera.phi -= (SPEED*delta_t) as f32 *self.mouse_motion.0;
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
        self.font.text(&format!("FPS {}", self.fps_counter.get()), 0., 0.12);
        self.font.update(&self.graphics);
        
        self.graphics.draw(
            |mut renderer| {
                // Update buffers
                for object in &self.objects {
                    object.copy_buffers(&mut renderer);
                }
                self.font.copy_buffers(&mut renderer);


                renderer.start();
                self.flat_shader.bind(&mut renderer);
                // Draw skybox
                
                renderer.clear();

                // Draw main game
                self.block_shader.bind(&mut renderer);
                self.camera.bind(&mut renderer);
                self.lighting.bind(&mut renderer);
                for object in &self.objects {
                    object.draw(&mut renderer, &self.texture)
                }

                renderer.clear();

                // Draw text
                self.font.render(&mut renderer, &self.camera, &self.lighting);
            },
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

struct FpsCounter {
    data: [f64; 64],
    cursor: usize,
}
impl FpsCounter {
    fn new() -> Self {
        Self {
            data: [-1.; 64],
            cursor: 0,
        }
    }

    fn update(&mut self, delta_t: f64) {
        self.data[self.cursor] = delta_t;
        self.cursor = (self.cursor + 1) % 64;
    }

    fn get(&self) -> i32 {
        let mut value = 0.;
        let mut total = 0;
        for item in &self.data {
            if *item == -1. {continue;}
            total += 1;
            value += item;
        }
        (total as f64 / value).round() as i32
    }
}