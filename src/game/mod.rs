pub mod object;
pub mod planet;
pub mod entity;
pub mod galaxy;
pub mod shading;

use std::cell::RefCell;
use std::rc::Rc;

use crate::game::entity::Entity;
use crate::game::galaxy::Galaxy;
use crate::game::object::ObjectLoader;
use crate::game::object::loader::ShipLoader;
use crate::game::shading::PostInfo;
use crate::graphics::*;
use crate::physics::*;
use crate::util::RcCell;
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
    block_shader: Shader,
    flat_shader: Shader,
    shadow_shader: Shader,
    post_shader: Shader,
    camera: Camera,
    texture: GridTexture,
    lighting: Lighting,
    font: Font,
    post_info: PostInfo,

    physics: Box<Physics>,
    objects: Vec<RcCell<Object>>,
    entities: Vec<Entity>,
    planets: Vec<Planet>,

    key_state: KeyState,
    fps_counter: FpsCounter,
    mouse_motion: (f32, f32),

    galaxy: Galaxy,
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
        ], false);
        let flat_shader = Shader::new::<FlatVertex>(&mut graphics, include_str!("../shaders/flat.wgsl"), vec![
            ResourceType::Camera,
            ResourceType::Model,
            ResourceType::Texture,
        ], false);
        let shadow_shader = Shader::new::<BlockVertex>(&mut graphics, include_str!("../shaders/shadow.wgsl"), vec![
            ResourceType::Camera,
            ResourceType::Model,
        ], false);
        let post_shader = Shader::new::<PostVertex>(&mut graphics, include_str!("../shaders/post.wgsl"), vec![
            ResourceType::Post,
            ResourceType::Texture,
            ResourceType::Camera,
            ResourceType::Lighting,
        ], true);
        let camera = Camera::new(&graphics);
        let lighting = Lighting::new(&graphics);
        let post_info = PostInfo::new(&graphics, &camera);

        let mut planets: Vec<Planet> = vec![
            // Planet::new(PlanetInit::default()),
        ];
        let mut objects = vec![
            Rc::new(RefCell::new(Object::new(&graphics, &mut physics, ObjectLoader::OneShot(ShipLoader{ pos: Vector3::new(12., 7., 43.), vel: Vector3::new(0., 0., 0.) })))),
        ];
        for planet in &mut planets {
            let obj = Rc::new(RefCell::new(Object::new(&graphics, &mut physics, planet.loader())));
            planet.object = Some(obj.clone());
            objects.push(obj);
        }
        let entities = vec![
            Entity::new(&mut physics, RigidBodyInit {pos: Vector3::new(0., 0., 50.), vel: Vector3::new(0., 0., 0.), ..Default::default()}),
        ];

        
        // Set cursor to center of screen
        let size = graphics.window.inner_size();
        let center = PhysicalPosition::new(
            size.width as f64 / 2.0,
            size.height as f64 / 2.0,
        );
        graphics.window.set_cursor_position(center).unwrap();

        // Load block texture
        let texture = GridTexture::new(&graphics, &camera, include_bytes!("../../assets/texture.png"));

        let font = Font::new(&mut graphics, &camera, include_bytes!("../../assets/Rockwell.ttc"));

        let mut galaxy = Galaxy::new(&graphics);
        galaxy.update_skybox(&graphics, &camera, Vector3::new(-1e7, 0., 0.));

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
            font,
            fps_counter: FpsCounter::new(),
            entities,
            galaxy,
            shadow_shader,
            post_shader,
            post_info,
            planets,
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
                let player = &mut self.entities[0];
                let body_collider = player.body.collider.take();
                let forward = self.camera.get_forward().cast().unwrap();
                player.body.collider = Some(Collider::new_ray(self.camera.pos.cast().unwrap(), forward*LOOK_DIST));

                let mut report = CollisionReport::None;
                let mut collided_object = None;

                for object in &mut self.objects {
                    // The collision function should always pick some over None, but choose the one with the smallest distance to the target otherwise.
                    let new_report = Collider::check_collision(&player.body, &object. borrow().body);

                    if new_report > report {
                        report = new_report;
                        collided_object = Some(object);
                    }
                }

                if let Some(o) = collided_object {
                    let place_pos = match report {
                        CollisionReport::Some { p2, .. } => {
                            let offset = o.borrow().body.ori.invert() * forward;
                            p2 - offset*0.001
                        }
                        CollisionReport::None => unreachable!(),
                    };
                    o.borrow_mut().insert_block(&self.graphics, 1, place_pos);
                }

                // Put the collider back
                player.body.collider = body_collider;
            },
            _ => (),
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        self.fps_counter.update(delta_t);
        for object in &mut self.objects {
            object.borrow_mut().update(&self.graphics, self.camera.pos.cast().unwrap());
        }

        let my_planet = if !self.planets.is_empty() {
            let mut min_dist = f64::INFINITY;
            let mut min_index = 0;
            for (i, planet) in self.planets.iter().enumerate() {
                if let Some(o) = &planet.object {
                    let o = o.borrow().body;
                    let dist = (o.pos - self.camera.pos.cast().unwrap()).magnitude();
                    if dist < min_dist {
                        min_index = i;
                        min_dist = dist;
                    }
                }
            }
            Some(&self.planets[min_index])
        } else {
            None
        };
        self.post_info.update_buffer(&self.graphics, &self.camera, my_planet);

        {
            // Move camera pos
            const SPEED: f64 = 1000.;
            let forward: Vector3<f64> = self.camera.get_forward().cast().unwrap();
            let up: Vector3<f64> = self.camera.get_up().cast().unwrap();
            let right: Vector3<f64> = self.camera.get_right().cast().unwrap();
            if self.key_state.get(KeyCode::KeyW) {
                self.entities[0].walk(forward * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyS) {
                self.entities[0].walk(-forward * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyD) {
                self.entities[0].walk(right * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyA){
                self.entities[0].walk(-right * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyQ) {
                self.entities[0].walk(up * (SPEED*delta_t));
            }
            if self.key_state.get(KeyCode::KeyE){
                self.entities[0].walk(-up * (SPEED*delta_t));
            }
        }
        
        self.physics.update(delta_t);

        {
            // Move camera look
            const SPEED: f64 = 0.2;
            self.camera.pos = self.entities[0].body.pos + 0.7f64 * Vector3::unit_z();
            self.camera.theta += (SPEED*delta_t) as f32 *self.mouse_motion.1;
            self.camera.phi -= (SPEED*delta_t) as f32 *self.mouse_motion.0;
            self.mouse_motion = (0., 0.);
            self.camera.theta = self.camera.theta.clamp(0.0001, 3.1415);
        }
    }

    pub fn draw(&mut self) {
        // OPTIMIZE avoid all calls of queue.write_buffer.
        self.camera.update_buffer(&self.graphics, &self.lighting, &self.camera);
        self.lighting.update_buffer(&self.graphics, &self.camera);
        for object in &mut self.objects {
            object.borrow_mut().update_buffer(&self.graphics, &self.camera)
        }
        self.font.text(&format!("FPS {}", self.fps_counter.get()), 0., 0.12);
        if !self.planets.is_empty() {
            self.font.text(&self.planets[0].dbg_text(self.camera.pos.cast().unwrap()), 0.0, 0.2);
        }
        self.font.update(&self.graphics);
        
        self.graphics.draw(
            |mut renderer| {
                // Update buffers
                for object in &self.objects {
                    object.borrow().copy_buffers(&mut renderer);
                }
                self.font.copy_buffers(&mut renderer);

                // Shadow render
                renderer.start_shadow(&mut self.camera);
                self.shadow_shader.bind(&mut renderer);
                self.camera.bind(&mut renderer);
                for object in &self.objects {
                    object.borrow().draw_shadow(&mut renderer);
                }

                // Sky box
                renderer.start();
                self.flat_shader.bind(&mut renderer);
                self.camera.bind(&mut renderer);
                self.galaxy.draw_skybox(&mut renderer);

                // Draw main game
                renderer.clear();
                self.block_shader.bind(&mut renderer);
                self.camera.bind(&mut renderer);
                self.lighting.bind(&mut renderer);
                for object in &self.objects {
                    object.borrow().draw(&mut renderer, &self.texture)
                }
                
                // Draw text
                self.font.render(&mut renderer, &self.camera, &self.lighting);

                // Finish
                renderer.start_post();
                self.post_shader.bind(&mut renderer);
                self.post_info.bind(&mut renderer);
                self.camera.bind(&mut renderer);
                self.lighting.bind(&mut renderer);
                renderer.stop_post();
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