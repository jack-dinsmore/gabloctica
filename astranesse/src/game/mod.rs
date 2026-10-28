pub mod object;
pub mod entity;
pub mod galaxy;
pub mod shading;
pub mod chemistry;

use crate::game::object::ObjectIdentifier;
use crate::graphics::*;
use crate::physics::*;
use crate::util::CHUNK_SIZE;
use crate::util::my_fmod;
use object::{Interrupt, BlockProperties, Planet, Ship};
use shading::PostInfo;
use galaxy::Galaxy;
use entity::Entity;
use cgmath::{InnerSpace, Vector3, Rotation};
use rustc_hash::FxHashSet;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey},
};

const LOOK_DIST: f64 = 5.;

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
    camera: Camera,
    texture: GridTexture,
    lighting: Lighting,
    font: Font,
    post_info: PostInfo,
    
    planets: Vec<Planet>,
    ships: Vec<Ship>,
    entities: Vec<Entity>,
    
    key_state: KeyState,
    fps_counter: FpsCounter,
    mouse_motion: (f32, f32),
    
    galaxy: Galaxy,
    block_properties: BlockProperties, // Needs to be last
    physics: Box<Physics>, // Needs to be last
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
        let camera = Camera::new(&graphics);
        let lighting = Lighting::new(&graphics);
        let post_info = PostInfo::new(&mut graphics, &camera);

        let planets = vec![
            Planet::new(&graphics, &mut physics),
        ];
        let ships = vec![
            Ship::new(&graphics, &mut physics, RigidBodyInit { pos: Vector3::new(12., 5., 83.), vel: Vector3::new(0., 0., 0.), ..Default::default() }),
        ];
        let entities = vec![
            // Entity::new(&mut physics, RigidBodyInit {pos: Vector3::new(-200., 0., 80.), vel: Vector3::new(0., 0., 0.), ..Default::default()}),
            Entity::new(&mut physics, RigidBodyInit {pos: Vector3::new(0., 0., 80.), vel: Vector3::new(0., 0., 0.), ..Default::default()}),
        ];

        // Set cursor to center of screen
        let size = graphics.window.inner_size();
        let center = PhysicalPosition::new(
            size.width as f64 / 2.0,
            size.height as f64 / 2.0,
        );
        graphics.window.set_cursor_position(center).unwrap();

        // Load block information
        let texture = GridTexture::new(&graphics, &camera, include_bytes!("../../../assets/texture.png"));
        let block_properties = BlockProperties::new();

        let font = Font::new(&mut graphics, &camera, include_bytes!("../../../assets/Rockwell.ttc"));

        let mut galaxy = Galaxy::new(&graphics);
        galaxy.update_skybox(&graphics, &camera, Vector3::new(-1e7, 0., 0.));

        Self {
            graphics,
            block_properties,
            key_state,
            camera,
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
            post_info,
            planets,
            ships,
        }
    }

    pub fn mouse_moved(&mut self, difference: (f32, f32)) {
        self.mouse_motion = difference;
    }
    
    fn mouse_clicked(&mut self, button: winit::event::MouseButton) {
        match button {
            winit::event::MouseButton::Left => {
                // Replace the player's collider with its look ray temporarily
                let player = &mut self.entities[0];
                let body_collider = player.body.collider.take();
                let forward = self.camera.get_forward().cast().unwrap();
                player.body.collider = Some(Collider::new_ray(self.camera.pos.cast().unwrap(), forward*LOOK_DIST));

                let mut report = CollisionReport::None;
                let mut collided_object = None;

                for planet in &mut self.planets {
                    let new_report = Collider::check_collision(&player.body, &planet.object.body);

                    if new_report > report {
                        report = new_report;
                        collided_object = Some(&mut planet.object);
                    }
                }
                for ship in &mut self.ships {
                    let new_report = Collider::check_collision(&player.body, &ship.object.body);

                    if new_report > report {
                        report = new_report;
                        collided_object = Some(&mut ship.object);
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
                    o.insert_block(&self.graphics, &self.block_properties, 1, place_pos);
                }

                // Put the collider back
                player.body.collider = body_collider;
            },
            winit::event::MouseButton::Right => self.interact(),
            _ => (),
        }
    }

    fn handle_keydowns(&mut self, delta_t: f64) {
        let player = &mut self.entities[0];
        match &player.chair {
            None => {
                // Move camera pos
                const SPEED: f64 = 1000.;
                let forward: Vector3<f64> = self.camera.get_forward().cast().unwrap();
                let up: Vector3<f64> = self.camera.get_up().cast().unwrap();
                let right: Vector3<f64> = self.camera.get_right().cast().unwrap();
                if self.key_state.get(KeyCode::KeyW) {
                    player.walk(forward * (SPEED*delta_t));
                }
                if self.key_state.get(KeyCode::KeyS) {
                    player.walk(-forward * (SPEED*delta_t));
                }
                if self.key_state.get(KeyCode::KeyD) {
                    player.walk(right * (SPEED*delta_t));
                }
                if self.key_state.get(KeyCode::KeyA){
                    player.walk(-right * (SPEED*delta_t));
                }
                if self.key_state.get(KeyCode::KeyQ) {
                    player.walk(up * (SPEED*delta_t));
                }
                if self.key_state.get(KeyCode::KeyE){
                    player.walk(-up * (SPEED*delta_t));
                }
            },
            Some((id, block_key, block_pos)) => {
                let object = id.get_object_mut(&mut self.ships, &mut self.planets);
                if self.key_state.get(KeyCode::KeyW) {
                    object.interrupt(*block_key, Interrupt::Forward(1.));
                }
                if self.key_state.get(KeyCode::KeyS) {
                    object.interrupt(*block_key, Interrupt::Backward(1.));
                }
                if self.key_state.get(KeyCode::KeyD) {
                    object.interrupt(*block_key, Interrupt::Right(1.));
                }
                if self.key_state.get(KeyCode::KeyA){
                    object.interrupt(*block_key, Interrupt::Left(1.));
                }
                if self.key_state.get(KeyCode::KeyQ) {
                    object.interrupt(*block_key, Interrupt::Up(1.));
                }
                if self.key_state.get(KeyCode::KeyE){
                    object.interrupt(*block_key, Interrupt::Down(1.));
                }
            }
        }
    }

    fn interact(&mut self) {
        // Replace the player's collider with its look ray temporarily
        let player = &mut self.entities[0];
        let body_collider = player.body.collider.take();
        let forward = self.camera.get_forward().cast().unwrap();
        player.body.collider = Some(Collider::new_ray(self.camera.pos.cast().unwrap(), forward*LOOK_DIST));

        let (collided_object, object_id, report) = {
            let mut report = CollisionReport::None;
            let mut object_id = ObjectIdentifier::Ship(0);
            let mut collided_object = None;
    
            for (i, ship) in self.ships.iter_mut().enumerate() {
                // The collision function should always pick some over None, but choose the one with the smallest distance to the target otherwise.
                let new_report = Collider::check_collision(&player.body, &ship.object.body);
    
                if new_report > report {
                    report = new_report;
                    collided_object = Some(&mut ship.object);
                    object_id = ObjectIdentifier::Ship(i);  
                }
            }
            for (i, planet) in self.planets.iter_mut().enumerate() {
                // The collision function should always pick some over None, but choose the one with the smallest distance to the target otherwise.
                let new_report = Collider::check_collision(&player.body, &planet.object.body);
    
                if new_report > report {
                    report = new_report;
                    collided_object = Some(&mut planet.object);
                    object_id = ObjectIdentifier::Planet(i);
                }
            }

            (collided_object, object_id, report)
        };

        if let Some(o) = collided_object {
            // Interacted with this block
            let place_pos = match report {
                CollisionReport::Some { p2, .. } => {
                    let offset = o.body.ori.invert() * forward;
                    p2 + offset*0.001
                }
                CollisionReport::None => unreachable!(),
            };
            let interacted_block = o.get_block(place_pos).clone();
            if !interacted_block.is_null() {
                let updated_chunk = (
                    (place_pos.x/CHUNK_SIZE as f64).floor() as i32,
                    (place_pos.y/CHUNK_SIZE as f64).floor() as i32,
                    (place_pos.z/CHUNK_SIZE as f64).floor() as i32,
                );
                let updated_block = (
                    my_fmod(place_pos.x, CHUNK_SIZE as f64) as u32,
                    my_fmod(place_pos.y, CHUNK_SIZE as f64) as u32,
                    my_fmod(place_pos.z, CHUNK_SIZE as f64) as u32,
                );
                let block_key = (updated_chunk, updated_block);
                o.interrupt(block_key, Interrupt::Interact);
                if self.block_properties[interacted_block.id].is_chair {
                    player.set_chair(object_id, block_key);
                }
            }
        }

        // Put the collider back
        player.body.collider = body_collider;
    }

    pub fn update(&mut self, delta_t: f64) {
        self.handle_keydowns(delta_t);
        
        let camera_pos = self.camera.pos.cast().unwrap();
        let my_planet = if !self.planets.is_empty() {
            let mut min_dist = f64::INFINITY;
            let mut min_index = 0;
            for (i, planet) in self.planets.iter().enumerate() {
                let o = planet.object.body.clone();
                let dist = (o.pos - camera_pos).magnitude();
                if dist < min_dist {
                    min_index = i;
                    min_dist = dist;
                }
            }
            Some(&self.planets[min_index])
        } else {
            None
        };
        self.post_info.update_buffer(&self.graphics, &self.camera, my_planet);
        
        for planet in &mut self.planets {
            planet.update(&self.graphics, &self.block_properties, camera_pos, delta_t);
        }
        for ship in &mut self.ships {
            ship.update(&self.graphics, &self.block_properties, camera_pos, delta_t);
        }
        for entity in &mut self.entities { // TODO reorder
            entity.update(delta_t, &self.ships, &self.planets);
        }
        
        self.physics.update(delta_t);
        
        // Update camera
        {
            let player = &self.entities[0];
            const SPEED: f64 = 0.2;
            self.camera.pos = player.body.pos + player.body.ori * Vector3::unit_z() * 0.5;
            self.camera.theta += (SPEED*delta_t) as f32 *self.mouse_motion.1;
            self.camera.phi -= (SPEED*delta_t) as f32 *self.mouse_motion.0;
            self.mouse_motion = (0., 0.);
            self.camera.theta = self.camera.theta.clamp(0.0001, 3.1415);
            self.camera.ori = player.body.ori.cast().unwrap();
        }
        self.fps_counter.update(delta_t);
    }

    pub fn draw(&mut self) {
        // OPTIMIZE avoid all calls of queue.write_buffer.
        self.camera.update_buffer(&self.graphics, &self.lighting, &self.camera);
        self.lighting.update_buffer(&self.graphics, &self.camera);
        for planet in &mut self.planets {
            planet.object.update_buffer(&self.graphics, &self.camera)
        }
        for ship in &mut self.ships {
            ship.object.update_buffer(&self.graphics, &self.camera)
        }
        
        self.font.text(&format!("FPS {}", self.fps_counter.get()), 0., 0.12);
        if !self.planets.is_empty() {
            self.font.text(&self.planets[0].dbg_text(self.camera.pos.cast().unwrap()), 0.0, 0.2);
        }
        self.font.update(&self.graphics);
        
        self.graphics.draw(
            |mut renderer| {
                // Update buffers
                for planet in &self.planets {
                    planet.object.copy_buffers(&mut renderer);
                }
                for ship in &self.ships {
                    ship.object.copy_buffers(&mut renderer);
                }
                self.font.copy_buffers(&mut renderer);

                // Shadow render
                renderer.start_shadow(&mut self.camera);
                self.shadow_shader.bind(&mut renderer);
                self.camera.bind(&mut renderer);
                for planet in &self.planets {
                    planet.object.draw_shadow(&mut renderer);
                }
                for ship in &self.ships {
                    ship.object.draw_shadow(&mut renderer);
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
                for planet in &self.planets {
                    planet.object.draw(&mut renderer, &self.texture)
                }
                for ship in &self.ships {
                    ship.object.draw(&mut renderer, &self.texture)
                }

                // Post
                self.post_info.render(&mut renderer, &self.camera, &self.lighting);
                
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