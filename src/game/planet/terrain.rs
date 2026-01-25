use core::f32;

use cgmath::{InnerSpace, Vector3};
use noise::{NoiseFn, Perlin};

use crate::{game::planet::{atmosphere::Atmosphere, ocean::Ocean}, util::SphericalInterpolator};


#[derive(Clone)]
pub struct Terrain {
    pub halfwidth: f32,
    weathering: f32,
    noise: Perlin,
    flow_interpolator: SphericalInterpolator<Vector3<f32>>,
    pub ocean_alt: Option<f32>,
}

impl Terrain {
    pub fn new(size: u32, seed: u32) -> Self {
        let flow_noise = [Perlin::new(seed+1), Perlin::new(seed+2), Perlin::new(seed+3)];

        let flow_interpolator = SphericalInterpolator::new(|pos| {
            let pa = [pos[0] as f64, pos[1] as f64, pos[2] as f64];
            let mut arrow = Vector3::new(
                flow_noise[0].get(pa) as f32,
                flow_noise[1].get(pa) as f32,
                flow_noise[2].get(pa) as f32
            );
            arrow /= 3.;
            arrow -= pos*arrow.dot(pos);
            arrow
        }, size as usize);
        Self {
            halfwidth: (size/2) as f32,
            weathering: 1.,
            noise: Perlin::new(seed),
            flow_interpolator,
            ocean_alt: None,
        }
    }

    fn get_divergence(&self, mut pos: Vector3<f32>) -> f32 {
        let dt = 0.1;
        let mut t = 0f32;
        let mut integral = 0.;
        for _ in 0..30 {
            pos /= pos.magnitude();
            let arrow = self.flow_interpolator.get(pos);
            let forward_vec = if arrow.magnitude2() > 0. {
                arrow.normalize() * dt
            } else {
                let v = pos.cross(Vector3::new(1., 1., 1.1));
                v.normalize() * dt
            };
            let right_vec = forward_vec.cross(pos).normalize()*dt;
            let forward = self.flow_interpolator.get(pos+forward_vec);
            let backward = self.flow_interpolator.get(pos-forward_vec);
            let left = self.flow_interpolator.get(pos+right_vec);
            let right = self.flow_interpolator.get(pos-right_vec);
            let divergence = ((forward - backward).dot(forward_vec) + (left - right).dot(right_vec))/dt;
            integral += divergence * (-t/self.weathering).exp();
            t += dt;
        }
        integral / dt / 20.
    }

    pub fn get_altitude(&self, mut pos: Vector3<f32>, face_index: u8) -> f32 {
        match face_index {
            0 => pos[0] = self.halfwidth as f32,
            1 => pos[0] = -self.halfwidth as f32,
            2 => pos[1] = self.halfwidth as f32,
            3 => pos[1] = -self.halfwidth as f32,
            4 => pos[2] = self.halfwidth as f32,
            5 => pos[2] = -self.halfwidth as f32,
            _ => unreachable!()
        }
        let posarray = [pos[0] as f64, pos[1] as f64, pos[2] as f64];
        0.5 + self.get_divergence(pos).max(0.001) * self.noise.get(posarray) as f32
    }

    pub fn get_interpolator(&self, resolution: usize) -> SphericalInterpolator<f32> {
        SphericalInterpolator::new(|pos| {
            let mut alt = -f32::INFINITY;
            for face_index in 0..6 {
                let chunk_alt = match face_index {
                    0 => pos.x - self.halfwidth,
                    1 => -pos.x - self.halfwidth,
                    2 => pos.y - self.halfwidth,
                    3 => -pos.y - self.halfwidth,
                    4 => pos.z - self.halfwidth,
                    5 => -pos.z - self.halfwidth,
                    _ => unreachable!()
                };
                alt = alt.max(chunk_alt + self.get_altitude(pos, face_index));
            }
            alt
        }, resolution)
    }
    
    pub(crate) fn set_ocean(&mut self, ocean: &Ocean) {
        self.ocean_alt = Some(ocean.alt)
    }
    
    pub(crate) fn weather(&mut self, atmosphere: &Atmosphere) {
        // TODO
    }
}