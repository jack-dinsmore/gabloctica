use faer::linalg::solvers::Solve;
use faer::{ColRef, Mat};

use cgmath::{InnerSpace, Vector3};
use noise::{NoiseFn, Perlin};

use crate::game::chemistry::{Chemical, Concentration, State};
use crate::game::object::BlockId;
use crate::game::object::{Object, ObjectLoader, loader::PlanetLoader};
use crate::physics::{BOLTZMANN, NEWTON_G, PROTON_MASS, STEFAN_BOLTZMANN};
use crate::util::{CHUNK_SIZE, RcCell, SphericalInterpolator};

const OCEAN_RESOLUTION: isize = 16;
const HUMIDITY_RESOLUTION: isize = 16;

fn newton_iteration(iterations: usize, x0: &[f64], func: impl Fn(&[f64]) -> Vec<f64>, hess: impl Fn(&[f64]) -> Mat<f64>) -> Vec<f64> {
    let mut x = ColRef::from_slice(x0).to_owned();
    for iter in 0..iterations {
        if x.iter().fold(false, |accum, c| accum || c.is_nan()) {
            panic!("Nan at iteration {iter}");
        }
        let x_owned: Vec<f64> = x.iter().copied().collect();
        let f = ColRef::from_slice(&func(&x_owned)).to_owned();
        let h = hess(&x_owned);
        let lu = h.full_piv_lu();
        x -= lu.solve(&f);
    }
    x.iter().map(|v| *v).collect::<Vec<_>>()
}


#[derive(Debug)]
pub struct Biome {
    typ: u8,
}

impl Biome {
    pub fn new(typ: u8) -> Self {
        Self {
            typ,
        }
    }

    pub fn get_block(&self, depth: i32) -> BlockId {
        match self.typ {
            0 => 1,
            1 => 2,
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlanetInit {
    width: u32,
    seed: u32,
    mass: f32,
    to_sun: Vector3<f32>,
    sun_flux: f32,
    spin_local: Vector3<f32>,
    composition: Vec<(Chemical, f32)>// Chemical, Abundance in Pa
}
impl Default for PlanetInit {
    fn default() -> Self {
        Self {
            width: 8,
            seed: 79842121,
            mass: 1000.,
            to_sun: Vector3::new(1., 0., 0.) * 10000.,
            spin_local: Vector3::new(0., 0., 0.1),
            sun_flux: 1e12,
            composition: vec![(Chemical::air(), 100_000.), (Chemical::water(), 5_000.)],
        }
    }
}

#[derive(Clone)]
pub struct Planet {
    pub width: u32,
    alt_interp: SphericalInterpolator<f32>,
    wind_interp: SphericalInterpolator<Vector3<f32>>,
    humidity_interp: SphericalInterpolator<f32>,
    temp_interp: SphericalInterpolator<f32>,
    ocean_alt: f32,
    pub halfwidth: f32,
    pub object: Option<RcCell<Object>>,
}
impl Planet {
    pub fn new(data: PlanetInit) -> Self {
        let mut seed = data.seed;

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
                ) / 5.;
                arrow -= pos*arrow.dot(pos); // Make normal
                arrow
            }, data.width as usize);

            let weathering = 1.;
            let alt_interp = SphericalInterpolator::new(|mut pos| {
                let original_pos = pos;
                let dt = 0.1;
                let mut impact = 1.;
                let mut integral = 0.;
                while impact > 0.01 {
                    let divergence = flow_interp.get_divergence(pos);
                    let arrow = flow_interp.get(pos);
                    integral += divergence * impact;
                    pos += arrow * dt;
                    impact *= 1. - dt*weathering;
                }
                let altitude = integral * dt * weathering;
                altitude
            }, data.width as usize);

            (flow_interp, alt_interp)
        };

        let equilib_t = data.sun_flux / (16. * std::f32::consts::PI * data.to_sun.magnitude2() * STEFAN_BOLTZMANN as f32);

        // Work out atmospheric properties
        let concentration = Concentration::new(&data.composition);
        let surface_states = concentration.get_states(equilib_t);
        let (scale_height, ocean_volume) = {
            let mut total_pressure = 0.;
            let mut partial_mm = 0.;
            let mut liquid_fractional_volume = 0.;
            for ((chemical, orig_pressure), state) in concentration.data.iter().zip(&surface_states) {
                let pressure = match state {
                    State::Solid => 0.,
                    State::Gas => *orig_pressure,
                    State::Liquid(p) => *p,
                };
                total_pressure += pressure;
                partial_mm += chemical.molecular_mass * pressure;

                if let State::Liquid(p) = state {
                    let liquid_density = (orig_pressure - p) * (chemical.molecular_mass * PROTON_MASS) / (BOLTZMANN * equilib_t);
                    liquid_fractional_volume += liquid_density / chemical.liquid_density;
                }
            }

            let mean_mm = partial_mm / total_pressure;
            let radius = (data.width * CHUNK_SIZE) as f32 / 2.;
            let g_accel = NEWTON_G as f32 * data.mass / radius.powi(2);
            let scale_height = BOLTZMANN * equilib_t / (PROTON_MASS * mean_mm * g_accel);
            let ocean_volume = 24. * radius*radius * scale_height * liquid_fractional_volume;

            (scale_height, ocean_volume)
        };

        // Work out ocean via Newton's method
        let ocean_alt = {
            let halfwidth = data.width as f32 / 2.;
            let mut alt = 0.;
            for _ in 0..4 {
                let da = (halfwidth*2. / OCEAN_RESOLUTION as f32).powi(2);
                let mut volume = 0.;
                let mut dvolume = 0.;
                for face_index in 0..6 {
                    for u in 0..OCEAN_RESOLUTION {
                        for v in 0..OCEAN_RESOLUTION {
                            let ux = u as f32 / OCEAN_RESOLUTION as f32;
                            let vx = v as f32 / OCEAN_RESOLUTION as f32;
                            let coord = match face_index {
                                0 => Vector3::new(halfwidth, ux, vx),
                                1 => Vector3::new(-halfwidth, ux, vx),
                                2 => Vector3::new(ux, halfwidth, vx),
                                3 => Vector3::new(ux, -halfwidth, vx),
                                4 => Vector3::new(ux, vx, halfwidth),
                                5 => Vector3::new(ux, vx, -halfwidth),
                                _ => unreachable!()
                            };
                            let this_alt = alt_interp.get(coord);
                            if this_alt < alt {
                                volume += (alt - this_alt) * da;
                                dvolume += da;
                            }
                        }
                    }
                }

                alt -= (ocean_volume - volume) / dvolume;
            };
            alt
        };
        
        let mut wind_interp = SphericalInterpolator::new(|pos| Vector3::new(0., 0., 0.), data.width as usize);
        let mut temp_interp = SphericalInterpolator::new(|pos| equilib_t, data.width as usize);
        for _ in 0..10 {
            // Update the ODEs
            // TODO
        }

        // Work out humidity
        let chemical = {
            let mut first_liquid = None;
            for ((chemical, _), state) in data.composition.iter().zip(surface_states) {
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
            }, data.width as usize)
        };

        Self {
            halfwidth: data.width as f32 / 2.,
            width: data.width,
            object: None,
            ocean_alt,
            alt_interp,
            wind_interp,
            humidity_interp,
            temp_interp,
        }
    }

    /// Get the altitude at a given coordinate in chunk units
    pub fn get_altitude(&self, pos: Vector3<f32>, face_index: u8) -> f32 {
        (0.5 + self.alt_interp.get(pos)).max(self.ocean_alt)
        
        // * self.noise.get(posarray) as f32;
    }

    pub fn get_biome(&self, mut pos: Vector3<f32>, face_index: u8) -> Biome {
        match face_index {
            0 => pos[0] = self.halfwidth as f32,
            1 => pos[0] = -self.halfwidth as f32,
            2 => pos[1] = self.halfwidth as f32,
            3 => pos[1] = -self.halfwidth as f32,
            4 => pos[2] = self.halfwidth as f32,
            5 => pos[2] = -self.halfwidth as f32,
            _ => unreachable!()
        }
        Biome::new(1)
    }

    pub fn loader(&self) -> ObjectLoader {
        ObjectLoader::MultiShot(PlanetLoader::new(self.width/2, &self))
    }
    
    pub fn dbg_text(&self, pos: Vector3<f32>) -> String {
        let max = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
        let face_index = if max == pos.x.abs() {
            if pos.x > 0. { 0 } else { 1 }
        } else if max == pos.y.abs() {
            if pos.y > 0. { 2 } else { 3 }
        } else {
            if pos.z > 0. { 4 } else { 5 }
        };
        let alt = self.get_altitude(pos, face_index);
        // let volume = self.atmosphere.get_ocean_volume();
        let biome = self.get_biome(pos, face_index);
        let humidity = self.humidity_interp.get(pos);
        let wind = self.wind_interp.get(pos);
        let temp = self.temp_interp.get(pos);
        format!("Altitude: {:.2}\nVolume: {}\nBiome: {:?}\nHumidity: {:.2}\nWind: {:?}\nTemp: {:.2}", alt, 1., biome, humidity, wind, temp)
    }
}