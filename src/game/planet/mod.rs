mod atmosphere;
mod ocean;
mod terrain;

pub use terrain::Terrain;
pub use atmosphere::Atmosphere;
pub use ocean::Ocean;

use cgmath::Vector3;

use crate::{game::object::{Object, ObjectLoader, loader::PlanetLoader}, util::RcCell};

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
    pub width: u32,
    terrain: Terrain,
    atmosphere: Atmosphere,
    pub object: Option<RcCell<Object>>,
}
impl Planet {
    pub fn new(data: PlanetInit) -> Self {
        let mut terrain = Terrain::new(data.width, data.seed);
        let mut atmosphere = Atmosphere::new(data.width, data.mass, data.to_sun, data.spin_local);
        let ocean = Ocean::new(&terrain, &atmosphere);
        terrain.set_ocean(&ocean);
        // atmosphere.set_flow(&terrain); // TODO
        // terrain.weather(&atmosphere); // TODO

        Self {
            width: data.width,
            terrain,
            atmosphere,
            object: None,
        }
    }
    pub fn loader(&self) -> ObjectLoader {
        ObjectLoader::MultiShot(PlanetLoader::new(self.width/2, &self.terrain, &self.atmosphere))
    }
    
    pub fn dbg_text(&self, pos: Vector3<f32>) -> String {
        let max = pos.x.abs().max(pos.y.abs().max(pos.z));
        let face_index = if max == pos.x.abs() {
            if pos.x > 0. { 0 } else { 1 }
        } else if max == pos.y.abs() {
            if pos.y > 0. { 2 } else { 3 }
        } else {
            if pos.z > 0. { 4 } else { 5 }
        };
        let alt = self.terrain.get_altitude(pos, face_index);
        let volume = self.atmosphere.get_ocean_volume();
        let biome = self.atmosphere.get_biome(pos, face_index);
        let humidity = self.atmosphere.get_humidity(pos);
        let wind = self.atmosphere.get_wind(pos);
        let temp = self.atmosphere.get_temp(pos);
        format!("Altitude: {:.2}\nVolume: {}\nBiome: {:?}\nHumidity: {:.2}\nWind: {:?}\nTemp: {:.2}", alt, volume, biome, humidity, wind, temp)
    }
}