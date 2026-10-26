use std::rc::Rc;

use faer::linalg::solvers::Solve;
use faer::{ColRef, Mat};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use cgmath::{InnerSpace, Vector3};
use noise::{NoiseFn, Perlin};

use crate::game::chemistry::{Chemical, Concentration, State};
use crate::game::object::BlockId;
use crate::game::object::{Object, ObjectLoader, loader::PlanetLoader};
use crate::physics::{BOLTZMANN, NEWTON_G, PROTON_MASS, STEFAN_BOLTZMANN};
use crate::util::{CHUNK_SIZE, RcCell, SphericalInterpolator, percentile_sorted};

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

#[derive(Clone)]
pub struct Biome {
    typ: u8,
    condition: Rc<Box<dyn Fn(f32, f32, f32) -> bool>> // TODO the RC is to make i able to be cloned, which might not be necessary in future
}

impl Biome {
    pub fn new(typ: u8, condition: impl Fn(f32, f32, f32) -> bool + 'static) -> Self {
        Self {
            condition: Rc::new(Box::new(condition)),
            typ,
        }
    }

    pub fn get_block(&self, depth: i32) -> BlockId {
        self.typ as BlockId
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
            composition: vec![(Chemical::air(), 100_000.), (Chemical::water(), 1.)],
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
    seed: u32,
    biomes: Vec<Biome>,
    pub halfwidth: f32,
    pub object: Option<RcCell<Object>>,

    //Aesthetic properties
    pub scale_height: f32,
    pub extinction: Vector3<f32>,
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
                );
                arrow -= pos*arrow.dot(pos); // Make normal
                arrow
            }, data.width as usize);

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
            }, data.width as usize);

            (flow_interp, alt_interp)
        };

        let equilib_t = data.sun_flux / (16. * std::f32::consts::PI * data.to_sun.magnitude2() * STEFAN_BOLTZMANN as f32);

        // Work out atmospheric properties
        let concentration = Concentration::new(&data.composition);
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

            let radius = (data.width * CHUNK_SIZE) as f32 / 2.;
            let g_accel = NEWTON_G as f32 * data.mass / radius.powi(2);
            let scale_height = BOLTZMANN * equilib_t / (PROTON_MASS * mean_mm * g_accel);
            let ocean_volume = 24. * radius*radius * scale_height * liquid_fractional_volume;
            extinction *= scale_height;

            (scale_height, ocean_volume, extinction)
        };

        // Work out ocean via Newton's method
        let ocean_alt = {
            let mut alt = 0.;
            for _ in 0..4 {
                let da = (data.width as f32 / OCEAN_RESOLUTION as f32).powi(2);
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

        let biomes = {
            let mut rng = StdRng::seed_from_u64(seed as u64);
            const N_TRIALS: usize = 64;
            let positions = (0..N_TRIALS).map(|_| Vector3::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))).collect::<Vec<_>>();
            let mut humidities = positions.iter().map(|pos| humidity_interp.get(*pos)).collect::<Vec<_>>();
            let mut temps = positions.iter().map(|pos| temp_interp.get(*pos)).collect::<Vec<_>>();
            let mut alts = positions.iter().map(|pos| alt_interp.get(*pos)).collect::<Vec<_>>();
            humidities.sort_by(|a, b| a.total_cmp(b));
            temps.sort_by(|a, b| a.total_cmp(b));
            alts.sort_by(|a, b| a.total_cmp(b));

            let upper_alt_thresh = percentile_sorted(&alts, 80.);

            let biomes = vec![
                Biome::new(2, move |alt,temp,humid| alt <= ocean_alt), // Ocean
                Biome::new(3, move |alt,temp,humid| alt > upper_alt_thresh), // High
                Biome::new(1, |_,_,_| true), // Normal
            ];

            biomes
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
            biomes,
            seed,
            scale_height,
            extinction,
        }
    }

    /// Get the altitude at a given coordinate in chunk units
    pub fn get_altitude(&self, pos: Vector3<f32>) -> f32 {
        8. + self.alt_interp.get(pos).max(self.ocean_alt)
        // * self.noise.get(posarray) as f32;
    }

    /// Get the distance between a given point and the surface
    pub fn get_point_altitude(&self, pos: Vector3<f32>) -> f32 {
        let max_distance = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
        max_distance - self.halfwidth * CHUNK_SIZE as f32
    }

    pub fn get_biome(&self, pos: Vector3<f32>) -> &Biome {
        let alt = self.alt_interp.get(pos);
        let temp = self.temp_interp.get(pos);
        let humid = self.humidity_interp.get(pos);
        for b in &self.biomes {
            if (b.condition)(alt, temp, humid) { return b; }
        }
        unreachable!()
    }

    pub fn loader(&self) -> ObjectLoader {
        ObjectLoader::MultiShot(PlanetLoader::new(self.width/2, &self))
    }
    
    pub fn dbg_text(&self, pos: Vector3<f32>) -> String {
        let alt = self.get_altitude(pos);
        let humidity = self.humidity_interp.get(pos);
        let wind = self.wind_interp.get(pos);
        let temp = self.temp_interp.get(pos);
        format!("Altitude: {:.2}\nVolume: {}\nHumidity: {:.2}\nWind: {:?}\nTemp: {:.2}", alt, 1., humidity, wind, temp)
    }
}