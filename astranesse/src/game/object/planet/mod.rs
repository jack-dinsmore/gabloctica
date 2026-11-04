mod textures;

use cgmath::{InnerSpace, Matrix4, Quaternion, Rotation, Vector3};
use noise::{NoiseFn, Perlin};
use rand::{RngExt, SeedableRng};
use rand::rngs::StdRng;
use textures::BlockSelector;

use crate::game::chemistry::{Chemical, Concentration, State};
use crate::game::object::chunk::PhysicalChunk;
use crate::game::object::{BlockProperties, RENDER_DISTANCE};
use crate::game::object::Object;
use crate::graphics::{Camera, GraphicalChunk, Graphics};
use crate::physics::{BOLTZMANN, MoI, NEWTON_G, PROTON_MASS, Physics, RigidBody, RigidBodyInit, STEFAN_BOLTZMANN};
use crate::util::{CHUNK_SIZE, Ori, SphericalInterpolator, get_face_index};

const OCEAN_RESOLUTION: isize = 16;
const HUMIDITY_RESOLUTION: isize = 16;
const LOAD_TIME: u128 = 25000; // Millseconds

struct LoadedPlanet {
    alt_interp: SphericalInterpolator<f32>,
    humidity_interp: SphericalInterpolator<f32>,
    temp_interp: SphericalInterpolator<f32>,
    ocean_alt: f32,
    rng: StdRng,
    last_load: std::time::Instant,
    halfwidth: f32,
    
    //Aesthetic properties
    scale_height: f32,
    seed: u64,
    extinction: Vector3<f32>,
    selector: BlockSelector,
}

pub struct Planet {
    width: u32,
    seed: u64,
    mass: f32,
    to_sun: Vector3<f32>,
    sun_flux: f32,
    spin_local: Vector3<f32>,
    concentration: Concentration,
    pub object: Object,

    loaded: Option<LoadedPlanet>,
}

impl LoadedPlanet {
    /// Get the altitude at a given coordinate in chunk units
    fn get_altitude(&self, pos: Vector3<f32>) -> f32 {
        8. + self.alt_interp.get(pos).max(self.ocean_alt)
        // * self.noise.get(posarray) as f32;
    }

    /// Get the distance between a given point and the surface
    fn get_point_altitude(&self, pos: Vector3<f32>) -> f32 {
        let max_distance = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
        max_distance - self.halfwidth
    }
    
    fn dbg_text(&self, pos: Vector3<f32>) -> String {
        let alt = self.get_altitude(pos);
        let humidity = self.humidity_interp.get(pos);
        let temp = self.temp_interp.get(pos);
        format!("Altitude: {:.2}\nVolume: {}\nHumidity: {:.2}\nTemp: {:.2}", alt, 1., humidity, temp)
    }

    pub(super) fn load_chunk(&mut self, graphics: &Graphics, properties: &BlockProperties, chunk_coord: (i32, i32, i32)) -> Option<PhysicalChunk> {

        let chunk_pos = Vector3::new(chunk_coord.0 as f32, chunk_coord.1 as f32, chunk_coord.2 as f32) * CHUNK_SIZE as f32;
        if self.get_point_altitude(chunk_pos) > self.get_altitude(chunk_pos) + CHUNK_SIZE as f32 {
            // The chunk is outside the planet. Don't load it.
            return None;
        }

        let mut chunk = PhysicalChunk::empty(graphics, chunk_coord);
        let face_index = get_face_index(chunk_pos);

        // Build the cube up from the bottom
        for u in 0..CHUNK_SIZE {
            for v in 0..CHUNK_SIZE {
                for w in 0..CHUNK_SIZE {
                    let pos = chunk_pos + Vector3::new(u as f32, v as f32, w as f32);
                    let depth = (self.get_altitude(pos) - self.get_point_altitude(pos)) as i32;
                    if depth < 0 { continue; } 

                    let alt = self.alt_interp.get(pos);
                    let temp = self.temp_interp.get(pos);
                    let humidity = self.humidity_interp.get(pos);
                    let id = self.selector.get_block_id(alt, humidity, temp);
                    let rot = Ori::random(&mut self.rng);
                    chunk.grid[(u,v,w)] = super::chunk::PhysicalBlock::new(id, rot, properties);
                }
            }
        }

        for u in -(CHUNK_SIZE as i32)..2*(CHUNK_SIZE as i32) {
            for v in -(CHUNK_SIZE as i32)..2*(CHUNK_SIZE as i32) {
                let mut seed = self.seed;
                let w = match face_index {
                    0|1 => {
                        let pos = chunk_pos + Vector3::new(0., u as f32, v as f32);
                        seed += (chunk_coord.1 * CHUNK_SIZE as i32 + u).cast_unsigned() as u64;
                        seed += ((chunk_coord.2 * CHUNK_SIZE as i32 + v).cast_unsigned() as u64) << 32;
                        (self.get_altitude(pos) - self.get_point_altitude(pos)) as i32 + 2
                    }
                    2|3 => {
                        let pos = chunk_pos + Vector3::new(u as f32, 0., v as f32);
                        seed += (chunk_coord.0 * CHUNK_SIZE as i32 + u).cast_unsigned() as u64;
                        seed += ((chunk_coord.2 * CHUNK_SIZE as i32 + v).cast_unsigned() as u64) << 32;
                        (self.get_altitude(pos) - self.get_point_altitude(pos)) as i32 + 2
                    }
                    4|5 => {
                        let pos = chunk_pos + Vector3::new(u as f32, v as f32, 0.);
                        seed += (chunk_coord.0 * CHUNK_SIZE as i32 + u).cast_unsigned() as u64;
                        seed += ((chunk_coord.1 * CHUNK_SIZE as i32 + v).cast_unsigned() as u64) << 32;
                        (self.get_altitude(pos) - self.get_point_altitude(pos)) as i32 + 2
                    },
                    _ => unreachable!()
                };
                if w < -(CHUNK_SIZE as i32) || w >= 2*CHUNK_SIZE as i32 { continue; }
                let (u, v, w) = match face_index {
                    0 => (w, u, v),
                    1 => (-w, u, v),
                    2 => (u, w, v),
                    3 => (u, -w, v),
                    4 => (u, v, w),
                    5 => (u, v, -w),
                    _ => unreachable!()
                };

                let mut rng = StdRng::seed_from_u64(seed);
                let pos = chunk_pos + Vector3::new(u as f32, v as f32, w as f32);
                if get_face_index(pos) != face_index {continue;};
                let global_ori = match face_index {
                    0 => Ori { data: 0 },
                    1 => Ori { data: 14 },
                    2 => Ori { data: 1 },
                    3 => Ori { data: 15 },
                    4 => Ori { data: 16 },
                    5 => Ori { data: 20 },
                    _ => unreachable!()
                } * Ori::z(rng.random::<u8>() % 4);

                
                let alt = self.alt_interp.get(pos);
                let temp = self.temp_interp.get(pos);
                let humidity = self.humidity_interp.get(pos);
                if let Some(structure) = self.selector.get_structure(alt, temp, humidity, &mut rng) {
                    structure.place(&mut chunk, (u, v, w), global_ori);
                }
            }
        }

        Some(chunk)
    }
    
    pub(super) fn unload_chunk(&self, chunk_coord: (i32, i32, i32), chunk: &PhysicalChunk) {
        // TODO implement unloading chunk with changes saved
    }
    
    fn get_post_params(&self, camera: &Camera, rb: &RigidBody) -> ([[f32; 4]; 4], [f32; 4]) {
        let stretch = 1. / self.halfwidth;
        let planet_model_inv = Matrix4::from_scale(stretch as f64) * Matrix4::from(rb.ori.invert()) * Matrix4::from_translation(camera.pos - rb.pos);
        let planet_model_inv = planet_model_inv.cast().unwrap().into();
        let fog = [self.extinction.x, self.extinction.y, self.extinction.z, self.scale_height * stretch];
        (planet_model_inv, fog)
    }
}


impl Planet {
    pub fn new(graphics: &Graphics, physics: &mut Physics) -> Self {
        let mass = 1000.;
        let width = 8;
        let real_width = width as f64 * CHUNK_SIZE as f64;

        let moi = mass as f64 * real_width.powi(2) / 6.;
        let rb_init = RigidBodyInit {
            mass: mass as f64,
            moi: MoI::new_diagonal(Vector3::new(1., 1., 1.)*moi),
            ..Default::default()
        };
        let object = Object::new(graphics, physics, rb_init);
        let concentration = Concentration::new(
            vec![Chemical::air(), Chemical::water()],
            vec![1.5e11, 0.5e10],
            real_width as f32,
            (NEWTON_G * mass as f64 / (real_width/2.).powi(2)) as f32
        );

        Self {
            seed: 79842121,
            width,
            mass,
            to_sun: Vector3::new(1., 0., 0.) * 10000.,
            spin_local: Vector3::new(0., 0., 0.1),
            sun_flux: 1e12,
            concentration,
            object,

            loaded: None,
        }
    }

    pub fn update(&mut self, graphics: &Graphics, properties: &mut BlockProperties, character_pos: Vector3<f64>, delta_t: f64) {
        self.object.update(delta_t);
        self.set_rigid_body();
        
        // Load/unload the entire planet
        let dist = (character_pos - self.object.body.pos).magnitude();
        if self.loaded.is_none() && dist < 2. * RENDER_DISTANCE as f64 * CHUNK_SIZE as f64 {
            self.load(graphics, properties);
        };
        if self.loaded.is_some() && dist > 2. * RENDER_DISTANCE as f64 * CHUNK_SIZE as f64 {
            self.loaded = None;
        };

        // Load chunks
        if let Some(loaded) = &mut self.loaded {
            if loaded.last_load.elapsed().as_millis() < LOAD_TIME { return; }
    
            // Get the current chunk
            let pos_body = self.object.body.ori.invert() * (character_pos - self.object.body.pos);
            let character_chunk = (
                pos_body.x as i32 / CHUNK_SIZE as i32,
                pos_body.y as i32 / CHUNK_SIZE as i32,
                pos_body.z as i32 / CHUNK_SIZE as i32
            );
            let collider = self.object.body.get_object_collider_mut();
    
            // Unload old chunks
            let keys = self.object.graphical_chunks.keys().copied().collect::<Vec<_>>();
            for coord in keys {
                let detail = get_detail(coord.0 - character_chunk.0, coord.1 - character_chunk.1,coord.2 - character_chunk.2,);
                if detail == 0 {
                    loaded.unload_chunk(coord, &self.object.physical_chunks[&coord]);
                    self.object.physical_chunks.remove(&coord);
                    self.object.graphical_chunks.remove(&coord);
                    collider.chunks.remove(&coord);
                } else {
                    self.object.graphical_chunks.get_mut(&coord).unwrap().set_detail(detail, graphics);
                }
            }
    
            // Load new chunks
            for dx in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                for dy in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                    for dz in (-RENDER_DISTANCE)..RENDER_DISTANCE {
                        let coord = (character_chunk.0 + dx, character_chunk.1 + dy, character_chunk.2 + dz);
                        if let None = self.object.physical_chunks.get(&coord) {
                            if let Some(mut chunk) = loaded.load_chunk(graphics, properties, coord) {
                                chunk.update_collider(coord, &mut self.object.body, properties);
                                let mut graphical_chunk = GraphicalChunk::from_physical(graphics, coord, &chunk, properties);
                                graphical_chunk.set_detail(get_detail(dx, dy, dz), graphics);
                                self.object.graphical_chunks.insert(coord, graphical_chunk);
                                self.object.physical_chunks.insert(coord, chunk);
                            }
                        }
                    }
                }
            }
    
            loaded.last_load = std::time::Instant::now();
        }
    }

    fn set_rigid_body(&mut self) {
        let fwidth = self.width as f64 * CHUNK_SIZE as f64;
        self.object.body.mass = self.mass as f64;
        self.object.body.pos = Vector3::new(0., 0., 0.);
        self.object.body.vel = Vector3::new(0., 0., 0.);
        self.object.body.ang_vel = Vector3::new(0., 0., 0.);
        self.object.body.ori = Quaternion::new(1., 0., 0., 0.);
        let moi = self.mass as f64 * fwidth * fwidth / 6.;
        self.object.body.moi = MoI::Diagonal { moi: Vector3::new(moi, moi, moi) };
    }
    
    fn load(&mut self, graphics: &Graphics, properties: &mut BlockProperties) {
        let mut rng = StdRng::seed_from_u64(self.seed);

        // Get continental layout
        let (continent_flow_interp, alt_interp) = {
            let flow_noise = [Perlin::new(rng.random()), Perlin::new(rng.random()), Perlin::new(rng.random())];
            const SCALE: f64 = 1.5;
            let flow_interp = SphericalInterpolator::new(|pos| {
                let pa = [pos[0] as f64 / SCALE, pos[1] as f64 / SCALE, pos[2] as f64 / SCALE];
                let mut arrow = Vector3::new(
                    flow_noise[0].get(pa) as f32,
                    flow_noise[1].get(pa) as f32,
                    flow_noise[2].get(pa) as f32
                );
                arrow -= pos*arrow.dot(pos); // Make normal
                arrow * 1.0
            }, self.width as usize);

            let weathering = 2.;
            let alt_interp = SphericalInterpolator::new(|mut pos| {
                let original_pos = pos;
                let dt = 0.1;
                let mut impact = 1.;
                let mut integral = 0.;
                while impact > 0.01 {
                    let divergence = flow_interp.divergence(pos).powi(3);
                    let arrow = flow_interp.get(pos);
                    integral += divergence * impact;
                    pos += arrow * dt;
                    impact *= 1. - dt*weathering;
                }
                let altitude = integral * dt;
                altitude
            }, self.width as usize);

            (flow_interp, alt_interp)
        };

        // Work out atmospheric properties
        let equilib_t = self.sun_flux / (16. * std::f32::consts::PI * self.to_sun.magnitude2() * STEFAN_BOLTZMANN as f32);
        let radius = (self.width * CHUNK_SIZE) as f32 / 2.;
        let g_accel = NEWTON_G as f32 * self.mass / radius.powi(2);

        let mut total_pressure = 0.;
        let mut mean_mm = 0.;
        let mut specific_heat = 0.;
        let mut extinction = Vector3::new(0., 0., 0.);
        let mut ocean_volume = 0.;
        let states = self.concentration.get_states(equilib_t);
        let mut liquid_chemical = None;
        for (state, chemical) in states.iter().zip(&self.concentration.chemicals) {
            let pressure = state.pressure();
            total_pressure += pressure;
            ocean_volume += state.volume();
            mean_mm += pressure * chemical.molecular_mass;
            extinction += pressure * chemical.opacity;
            specific_heat += pressure * chemical.specific_heat;
            if let State::Liquid(_, _) = state {
                liquid_chemical = Some(chemical.clone());
            }
        }
        mean_mm /= total_pressure;
        specific_heat /= total_pressure;

        let scale_height = BOLTZMANN * equilib_t / (PROTON_MASS * mean_mm * g_accel);
        extinction *= scale_height;
        let eta = (total_pressure * specific_heat / (STEFAN_BOLTZMANN as f32 * equilib_t.powi(3) * (g_accel*scale_height).sqrt())).tanh();

        // Work out ocean via Newton's method
        let ocean_alt = {
            let mut alt = 0.;
            for _ in 0..4 {
                let da = (self.width as f32 / OCEAN_RESOLUTION as f32).powi(2);
                let mut volume = 0.;
                let mut dvolume = 0.;
                let mut min_alt = 0f32;
                for face_index in 0..6 {
                    for u in 0..OCEAN_RESOLUTION {
                        for v in 0..OCEAN_RESOLUTION {
                            let ux = u as f32 / OCEAN_RESOLUTION as f32;
                            let vx = v as f32 / OCEAN_RESOLUTION as f32;
                            let coord = match face_index {
                                0 => Vector3::new(1., ux, vx),
                                1 => Vector3::new(0., ux, vx),
                                2 => Vector3::new(ux, 1., vx),
                                3 => Vector3::new(ux, 0., vx),
                                4 => Vector3::new(ux, vx, 1.),
                                5 => Vector3::new(ux, vx, 0.),
                                _ => unreachable!()
                            };
                            let this_alt = alt_interp.get(coord);
                            min_alt = min_alt.min(this_alt);
                            if this_alt <= alt {
                                volume += (alt - this_alt) * da;
                                dvolume += da;
                            }
                        }
                    }
                }
                if dvolume == 0. {
                    alt = min_alt;
                } else {
                    alt -= (volume - ocean_volume) / dvolume;
                }
            };
            alt
        };
        
        // Work out the average temperature
        let temp_interp = SphericalInterpolator::new(|pos| {
            let max = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
            let mut local_normal = if max == pos.x {
                Vector3::new(1., 0., 0.)
            } else if max == -pos.x {
                Vector3::new(-1., 0., 0.)
            } else if max == pos.y {
                Vector3::new(0., 1., 0.)
            } else if max == -pos.y {
                Vector3::new(0., -1., 0.)
            } else if max == pos.z {
                Vector3::new(0., 0., 1.)
            } else {
                Vector3::new(0., 0., -1.)
            };
            local_normal += alt_interp.gradient(pos);
            local_normal = local_normal.normalize();
            let spin_axis = self.spin_local.normalize();
            let sun_axis = self.to_sun.normalize();

            let ndot2 = local_normal.dot(spin_axis).powi(2);
            let sdot2 = sun_axis.dot(spin_axis).powi(2) / 2.;

            let temp = equilib_t * 3. / 2. * (1. - ndot2 * (1. - 3.*sdot2) - sdot2).powf(0.25);
            temp * (1. - eta) + equilib_t * eta
        }, self.width as usize);

        // Work out the average wind flow
        let force_interp = SphericalInterpolator::new(|pos| {
            g_accel * scale_height * (temp_interp.gradient(pos) / equilib_t - alt_interp.gradient(pos) / scale_height)
        }, self.width as usize);
        let mut wind_interp = SphericalInterpolator::new(|pos| Vector3::new(0., 0., 0.), self.width as usize);
        for _ in 0..10 {
            // Update the ODEs
            wind_interp = SphericalInterpolator::new(|pos| {
                let wind = wind_interp.get(pos);
                let force = force_interp.get(pos) + 2. * self.spin_local.cross(wind);
                let residuals = wind_interp.convective_derivative(pos) - force;
                wind - residuals * 0.01
            }, self.width as usize);

        }

        // Work out humidity
        let humidity_interp = match liquid_chemical {
            Some(c) => {
                let surface_vapor_pressure = c.get_vapor_pressure(equilib_t);
                SphericalInterpolator::new(|mut pos| {
                    let mut humidity = surface_vapor_pressure;
                    let rain_rate = 1.;
                    let dt = 0.1;
                    while humidity > surface_vapor_pressure / 100. {
                        pos /= pos.magnitude();
                        let alt = alt_interp.get(pos);
                        let surface_temp = temp_interp.get(pos);
                        let condensation_pressure = c.get_vapor_pressure(surface_temp * (-alt/scale_height).exp());
                        if condensation_pressure < humidity {
                            // Condensed 
                            humidity = condensation_pressure;
                        };
                        pos -= wind_interp.get(pos) * dt;
                        humidity *= 1. - rain_rate * dt;
                        if alt_interp.get(pos) < ocean_alt {
                            break;
                        }
                    }
                    humidity
                }, self.width as usize)
            },
            None => {
                SphericalInterpolator::new(|pos| 0., self.width as usize)
            },
        };

        let selector = BlockSelector::new(properties, &alt_interp, &temp_interp, &humidity_interp, &mut rng);

        self.loaded = Some(LoadedPlanet {
            alt_interp,
            humidity_interp,
            temp_interp,
            ocean_alt,
            rng,
            scale_height,
            extinction,
            halfwidth: self.width as f32 * CHUNK_SIZE as f32 / 2.,
            last_load: std::time::Instant::now() - std::time::Duration::from_millis(2*LOAD_TIME as u64),
            selector,
            seed: self.seed,
        });
    }
    
    pub fn dbg_text(&self, pos: Vector3<f32>) -> String {
        match &self.loaded {
            Some(l) => l.dbg_text(pos),
            None => "".to_owned(),
        }
    }

    /// Return the fog parameters. Order of return statement: planet_model_inv, fog.
    pub fn get_post_params(&self, camera: &Camera) -> Option<([[f32; 4]; 4], [f32; 4])> {
        match &self.loaded {
            Some(l) => Some(l.get_post_params(camera, &self.object.body)),
            None => None
        }
    }
}

/// Return the detail of the cube given how far away it is
fn get_detail(dx: i32, dy: i32, dz: i32) -> usize {
    let dist = dx.abs().max(dy.abs().max(dz.abs()));
    match dist {
        0..8 => 1,
        8..12 => 2,
        // 12..RENDER_DISTANCE => 4,
        _ => 0, // Unloaded
    }
}