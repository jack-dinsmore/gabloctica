use cgmath::{InnerSpace, Matrix4, Quaternion, Rotation, Vector3};
use noise::{NoiseFn, Perlin};

use crate::game::chemistry::{Chemical, Concentration, State};
use crate::game::object::chunk::PhysicalChunk;
use crate::game::object::{BlockProperties, RENDER_DISTANCE};
use crate::game::object::Object;
use crate::graphics::{Camera, GraphicalChunk, Graphics};
use crate::physics::{BOLTZMANN, MoI, NEWTON_G, PROTON_MASS, Physics, RigidBody, RigidBodyInit, STEFAN_BOLTZMANN};
use crate::util::{CHUNK_SIZE, SphericalInterpolator};

const OCEAN_RESOLUTION: isize = 16;
const HUMIDITY_RESOLUTION: isize = 16;
const LOAD_TIME: u128 = 25000; // Millseconds

struct LoadedPlanet {
    alt_interp: SphericalInterpolator<f32>,
    wind_interp: SphericalInterpolator<Vector3<f32>>,
    humidity_interp: SphericalInterpolator<f32>,
    temp_interp: SphericalInterpolator<f32>,
    ocean_alt: f32,
    seed: u32,
    last_load: std::time::Instant,
    halfwidth: f32,

    //Aesthetic properties
    pub scale_height: f32,
    pub extinction: Vector3<f32>,
}

pub struct Planet {
    width: u32,
    seed: u32,
    mass: f32,
    to_sun: Vector3<f32>,
    sun_flux: f32,
    spin_local: Vector3<f32>,
    composition: Vec<(Chemical, f32)>, // Chemical, Abundance in Pa
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
        let wind = self.wind_interp.get(pos);
        let temp = self.temp_interp.get(pos);
        format!("Altitude: {:.2}\nVolume: {}\nHumidity: {:.2}\nWind: {:?}\nTemp: {:.2}", alt, 1., humidity, wind, temp)
    }

    pub(super) fn load_chunk(&self, graphics: &Graphics, properties: &BlockProperties, chunk_coord: (i32, i32, i32)) -> Option<PhysicalChunk> {
        let chunk_pos = Vector3::new(chunk_coord.0 as f32, chunk_coord.1 as f32, chunk_coord.2 as f32) * CHUNK_SIZE as f32;
        if self.get_point_altitude(chunk_pos) > self.get_altitude(chunk_pos) + CHUNK_SIZE as f32 {
            // The chunk is outside the planet. Don't load it.
            return None;
        }

        let mut chunk = PhysicalChunk::empty(graphics, chunk_coord);
        let max = chunk_pos.x.abs().max(chunk_pos.y.abs().max(chunk_pos.z.abs()));
        let face_index = if max == chunk_pos.x.abs() {
            if chunk_pos.x > 0. { 0 } else { 1 }
        } else if max == chunk_pos.y.abs() {
            if chunk_pos.y > 0. { 2 } else { 3 }
        } else {
            if chunk_pos.z > 0. { 4 } else { 5 }
        };

        // Build the cube up from the bottom
        for u in 0..CHUNK_SIZE {
            for v in 0..CHUNK_SIZE {
                for w in 0..CHUNK_SIZE {
                    let pos = chunk_pos + Vector3::new(u as f32, v as f32, w as f32);
                    let depth = (self.get_altitude(pos) - self.get_point_altitude(pos)) as i32;
                    if depth < 0 {continue;}
                    let block = 3;//biome.get_block(depth);//TODO
                    chunk.grid[(u,v,w)] = super::chunk::PhysicalBlock::new(1, 0, properties);
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

        let moi = mass as f64 * (width as f64 * CHUNK_SIZE as f64).powi(2) / 6.;
        let rb_init = RigidBodyInit {
            mass: mass as f64,
            moi: MoI::new_diagonal(Vector3::new(1., 1., 1.)*moi),
            ..Default::default()
        };
        let object = Object::new(graphics, physics, rb_init);

        Self {
            seed: 79842121,
            width,
            mass,
            to_sun: Vector3::new(1., 0., 0.) * 10000.,
            spin_local: Vector3::new(0., 0., 0.1),
            sun_flux: 1e12,
            composition: vec![(Chemical::air(), 100_000.), (Chemical::water(), 1.)],
            object,

            loaded: None,
        }
    }

    pub fn update(&mut self, graphics: &Graphics, properties: &BlockProperties, character_pos: Vector3<f64>, delta_t: f64) {
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

        // Load/unload chunks
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
                    dbg!();
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
                                chunk.update_collider(coord, &mut self.object.body);
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
    
    fn load(&mut self, graphics: &Graphics, properties: &BlockProperties) {
        let mut seed = self.seed;

        // Get continental layout
        let (continent_flow_interp, alt_interp) = {
            let flow_noise = [Perlin::new(seed+1), Perlin::new(seed+2), Perlin::new(seed+3)];
            seed += 3;
            let flow_interp = SphericalInterpolator::new(|pos| {
                let pa = [pos[0] as f64, pos[1] as f64, pos[2] as f64];
                let mut arrow = Vector3::new(
                    flow_noise[0].get(pa) as f32,
                    flow_noise[1].get(pa) as f32,
                    flow_noise[2].get(pa) as f32
                );
                arrow -= pos*arrow.dot(pos); // Make normal
                arrow
            }, self.width as usize);

            let weathering = 2.;
            let alt_interp = SphericalInterpolator::new(|mut pos| {
                let original_pos = pos;
                let dt = 0.1;
                let mut impact = 1.;
                let mut integral = 0.;
                while impact > 0.01 {
                    let divergence = flow_interp.get_divergence(pos).powi(3);
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

        let equilib_t = self.sun_flux / (16. * std::f32::consts::PI * self.to_sun.magnitude2() * STEFAN_BOLTZMANN as f32);

        // Work out atmospheric properties
        let concentration = Concentration::new(&self.composition);
        let surface_states = concentration.get_states(equilib_t);
        let (scale_height, ocean_volume, extinction) = {
            let mut total_pressure = 0.;
            let mut mean_mm = 0.;
            let mut extinction = Vector3::new(0., 0., 0.);
            let mut liquid_fractional_volume = 0.;
            for ((chemical, orig_pressure), state) in concentration.data.iter().zip(&surface_states) {
                let pressure = match state {
                    State::Solid => 0.,
                    State::Gas => *orig_pressure,
                    State::Liquid(p) => *p,
                };
                total_pressure += pressure;
                mean_mm += chemical.molecular_mass * pressure;
                extinction += pressure * chemical.opacity;

                if let State::Liquid(p) = state {
                    let liquid_density = (orig_pressure - p) * (chemical.molecular_mass * PROTON_MASS) / (BOLTZMANN * equilib_t);
                    liquid_fractional_volume += liquid_density / chemical.liquid_density;
                }
            }
            liquid_fractional_volume /= total_pressure;
            mean_mm /= total_pressure;

            let radius = (self.width * CHUNK_SIZE) as f32 / 2.;
            let g_accel = NEWTON_G as f32 * self.mass / radius.powi(2);
            let scale_height = BOLTZMANN * equilib_t / (PROTON_MASS * mean_mm * g_accel);
            let ocean_volume = 24. * radius*radius * scale_height * liquid_fractional_volume;
            extinction *= scale_height;

            (scale_height, ocean_volume, extinction)
        };

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
        
        let mut wind_interp = SphericalInterpolator::new(|pos| Vector3::new(0., 0., 0.), self.width as usize);
        let mut temp_interp = SphericalInterpolator::new(|pos| equilib_t, self.width as usize);
        for _ in 0..10 {
            // Update the ODEs
            // TODO
        }

        // Work out humidity
        let chemical = {
            let mut first_liquid = None;
            for ((chemical, _), state) in self.composition.iter().zip(surface_states) {
                if let State::Liquid(_) = state {
                    first_liquid = Some(chemical);
                    break; 
                }
            }
            first_liquid.unwrap()
        };
        let humidity_interp = {
            let surface_vapor_pressure = chemical.get_pressure(equilib_t);
            SphericalInterpolator::new(|mut pos| {
                let mut humidity = surface_vapor_pressure;
                let rain_rate = 1.;
                let dt = 0.1;
                while humidity > surface_vapor_pressure / 100. {
                    pos /= pos.magnitude();
                    let alt = alt_interp.get(pos);
                    let surface_temp = temp_interp.get(pos);
                    let condensation_pressure = chemical.get_pressure(surface_temp * (-alt/scale_height).exp());
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
        };

        self.loaded = Some(LoadedPlanet {
            alt_interp,
            wind_interp,
            humidity_interp,
            temp_interp,
            ocean_alt,
            seed,
            scale_height,
            extinction,
            halfwidth: self.width as f32 * CHUNK_SIZE as f32 / 2.,
            last_load: std::time::Instant::now() - std::time::Duration::from_millis(2*LOAD_TIME as u64),
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