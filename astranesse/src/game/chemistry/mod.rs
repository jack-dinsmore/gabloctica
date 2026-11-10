use cgmath::Vector3;
use itertools::izip;

use crate::util::lerp;

#[derive(Clone, Debug)]
pub struct Chemical {
    pub critical: (f32, f32), // Temperature, pressure
    pub triple: (f32, f32), // Temperature, pressure
    pub molecular_mass: f32,
    pub liquid_density: f32,
    pub opacity: Vector3<f32>,
    pub specific_heat: f32, // Extinction per pascal (1 / m / Pa) in r, g, b
}

impl Chemical {
    pub fn water() -> Self {
        Self {
            critical: (600., 1e3),
            triple: (270., 1e-1),
            molecular_mass: 18.,
            liquid_density: 1.,
            specific_heat: 0.004,
            opacity: Vector3::new(0.3, 0.5, 1.) * 1.,
        }
    }
    pub fn air() -> Self {
        Self {
            critical: (300., 3e6),
            triple: (70., 3e1),
            molecular_mass: 18.,
            liquid_density: 1.,
            specific_heat: 0.,
            opacity: Vector3::new(0., 0., 0.),
        }
    }

    pub fn get_vapor_pressure(&self, temp: f32) -> f32 {
        if temp < self.triple.0 {
            lerp(temp.ln(), 0., self.triple.0.ln(), 0., self.triple.1.ln()).exp()
        } else {
            lerp(temp.ln(), self.triple.0.ln(), self.critical.0.ln(), self.triple.1.ln(), self.critical.1.ln()).exp()
        }
    }
}


/// Struct to store information that can be used to compute how a system of gases behaves
#[derive(Clone, Copy, Debug)]
pub enum State {
    Gas(f32),
    Solid(f32, f32), // Solid volume, Vapor pressure
    Liquid(f32, f32), // Liquid volume, Vapor pressure
}
impl State {
    pub fn pressure(&self) -> f32 {
        match self {
            State::Gas(p) => *p,
            State::Solid(_, p) => *p,
            State::Liquid(_, p) => *p,
        }
    }
    pub fn volume(&self) -> f32 {
        match self {
            State::Gas(_) => 0.,
            State::Solid(v, _) => *v,
            State::Liquid(v, _) => *v,
        }
    }
}
pub struct Concentration {
    pub chemicals: Vec<Chemical>,
    masses: Vec<f32>,
    pressures: Vec<f32>,
}
impl Concentration {
    /// make an atmospheric concentration, providing the masses of each chemical
    pub fn new(chemicals: Vec<Chemical>, masses: Vec<f32>, w: f32, g: f32) -> Self {
        let pressures = masses.iter().map(|m| m*g / (6.*w*w)).collect::<Vec<f32>>();
        Self {
            chemicals,
            masses,
            pressures,
        }
    }

    pub fn get_states(&self, temp: f32) -> Vec<State> {
        let mut output = Vec::new();
        for (chemical, mass, pressure) in izip!(&self.chemicals, &self.masses, &self.pressures) {
            let vapor_pressure = chemical.get_vapor_pressure(temp);
            if vapor_pressure > *pressure {
                output.push(State::Gas(*pressure));
            } else {
                let condensed_mass = mass * (1. - vapor_pressure / *pressure);
                if temp < chemical.triple.0 {
                    output.push(State::Solid(condensed_mass / chemical.liquid_density, vapor_pressure));
                } else {
                    output.push(State::Liquid(condensed_mass / chemical.liquid_density, vapor_pressure));
                }
            }
        }
        output
    }
}