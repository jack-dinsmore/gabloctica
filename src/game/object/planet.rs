use noise::Perlin;

use crate::game::object::{ObjectLoader, loader::PlanetLoader};

pub struct PlanetInit {
    width: u32,
    seed: u32,
}
impl Default for PlanetInit {
    fn default() -> Self {
        Self {
            width: 16,
            seed: 79842121,
        }
    }
}

pub struct Planet {
    width: u32,
    seed: u32,
}
impl Planet {
    pub fn new(data: PlanetInit) -> Self {
        Self {
            width: data.width,
            seed: data.seed,
        }
    }
    pub fn loader(&self) -> ObjectLoader {
        let noise = Perlin::new(self.seed);
        ObjectLoader::MultiShot(PlanetLoader::new(self.width/2, noise))
    }
}