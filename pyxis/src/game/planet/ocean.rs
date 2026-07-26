use cgmath::Vector3;

use crate::{game::planet::{atmosphere::Atmosphere, terrain::Terrain}, util::SphericalInterpolator};

const RESOLUTION: usize = 16;

pub struct Ocean {
    pub alt: f32,
}
impl Ocean {
    pub fn new(terrain: &Terrain, atmosphere: &Atmosphere) -> Self {
        let volume = atmosphere.get_ocean_volume();
        let mut alt = 0.;
        let interp = terrain.get_interpolator(RESOLUTION);
        for _ in 0..4 {
            let (vol, vol_deriv) = Self::get_volume(&interp, terrain.halfwidth, alt);
            alt -= (vol - volume) / vol_deriv;
        };
        Self {
            alt
        }
    }

    /// Gets the volume covered by a given altitude, and its derivative
    pub fn get_volume(interp: &SphericalInterpolator<f32>, halfwidth: f32, alt: f32) -> (f32, f32) {
        let da = (halfwidth*2. / RESOLUTION as f32).powi(2);
        let mut volume = 0.;
        let mut dvolume = 0.;
        for face_index in 0..6 {
            for u in 0..RESOLUTION {
                let ux = u as f32 / RESOLUTION as f32;
                for v in 0..RESOLUTION {
                    let vx = v as f32 / RESOLUTION as f32;
                    let coord = match face_index {
                        0 => Vector3::new(halfwidth, ux, vx),
                        1 => Vector3::new(-halfwidth, ux, vx),
                        2 => Vector3::new(ux, halfwidth, vx),
                        3 => Vector3::new(ux, -halfwidth, vx),
                        4 => Vector3::new(ux, vx, halfwidth),
                        5 => Vector3::new(ux, vx, -halfwidth),
                        _ => unreachable!()
                    };
                    let this_alt = interp.get(coord);
                    if this_alt < alt {
                        volume += (alt - this_alt) * da;
                        dvolume += da;
                    }
                }
            }
        }
        (volume, dvolume)
    }
}