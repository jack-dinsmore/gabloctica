mod atmosphere;
mod ocean;
mod terrain;

pub use terrain::Terrain;
pub use atmosphere::Atmosphere;
pub use ocean::Ocean;

use cgmath::Vector3;

use crate::game::object::{ObjectLoader, loader::PlanetLoader};

pub struct PlanetInit {
    width: u32,
    seed: u32,
    mass: f32,
    to_sun: Vector3<f32>,
    spin_local: Vector3<f32>,
}
impl Default for PlanetInit {
    fn default() -> Self {
        Self {
            width: 4,
            seed: 79842121,
            mass: 1000.,
            to_sun: Vector3::new(1., 0., 0.) * 0.001,
            spin_local: Vector3::new(0., 0., 0.1),
        }
    }
}

pub struct Planet {
    width: u32,
    terrain: Terrain,
    atmosphere: Atmosphere,
}
impl Planet {
    pub fn new(data: PlanetInit) -> Self {
        let mut terrain = Terrain::new(data.width, data.seed);
        let mut atmosphere = Atmosphere::new(data.width, data.mass, data.to_sun, data.spin_local);
        let ocean = Ocean::new(&terrain, &atmosphere);
        terrain.set_ocean(&ocean);
        atmosphere.set_flow(&terrain);
        terrain.weather(&atmosphere);

        Self {
            width: data.width,
            terrain,
            atmosphere,
        }
    }
    pub fn loader(&self) -> ObjectLoader {
        ObjectLoader::MultiShot(PlanetLoader::new(self.width/2, &self.terrain, &self.atmosphere))
    }
}