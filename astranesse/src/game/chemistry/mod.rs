use crate::util::lerp;

#[derive(Clone, Debug)]
pub struct Chemical {
    pub critical: (f32, f32), // Temperature, pressure
    pub triple: (f32, f32), // Temperature, pressure
    pub molecular_mass: f32,
    pub liquid_density: f32,
}

impl Chemical {
    pub fn water() -> Self {
        Self {
            critical: (1000., 1_000_000.),
            triple: (100., 10.),
            molecular_mass: 18.,
            liquid_density: 1.,
        }
    }
    pub fn air() -> Self {
        Self {
            critical: (100., 1_000_000.),
            triple: (10., 10.),
            molecular_mass: 18.,
            liquid_density: 1.,
        }
    }

    pub fn get_pressure(&self, temp: f32) -> f32 {
        if temp < self.triple.0 {
            0.
        } else {
            lerp(temp.ln(), self.triple.0.ln(), self.critical.0.ln(), self.triple.1.ln(), self.critical.1.ln()).exp()
        }
    }
}


/// Struct to store information that can be used to compute how a system of gases behaves
#[derive(Clone, Copy, Debug)]
pub enum State {
    Solid,
    Gas,
    Liquid(f32), // Partial papor pressure
}
pub struct Concentration {
    pub data: Vec<(Chemical, f32)>,
}
impl Concentration {
    pub fn new(data: &[(Chemical, f32)]) -> Self {
        let mut data = (*data).to_vec();
        data.sort_by(|a, b| (a.0.triple.0).partial_cmp(&b.0.triple.0).unwrap());
        Self {
            data: data.to_vec(),
        }
    }

    pub fn get_states(&self, temp: f32) -> Vec<State> {
        let mut total_pressure = 0.;
        let mut output = Vec::new();
        for (chemical, pressure) in &self.data {
            if chemical.triple.0 > temp {
                output.push(State::Solid);
                continue;
            } else {
                // It's a liquid or gas
                let vp = chemical.get_pressure(temp);
                if total_pressure > vp {
                    output.push(State::Liquid(0.));
                    continue
                } else if total_pressure + pressure < vp {
                    output.push(State::Gas);
                    total_pressure += pressure;
                } else {
                    let added_pressure = vp - total_pressure;
                    total_pressure = vp;
                    output.push(State::Liquid(added_pressure));
                }
            }
        }
        output
    }
}