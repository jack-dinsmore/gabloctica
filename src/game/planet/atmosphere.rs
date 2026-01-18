use cgmath::{InnerSpace, Vector3};
use faer::{ColRef, Mat, prelude::Solve};

use crate::{game::planet::terrain::Terrain, util::SphericalInterpolator};

/// Scale height of mountains deflecting wind around them
const MOUNTAIN_SCALE: f32 = 1.;

pub struct Atmosphere {
    wind_interp: SphericalInterpolator<Vector3<f32>>,
    humidity_interp: SphericalInterpolator<f32>,
    temp_interp: SphericalInterpolator<f32>,
    resolution: usize,

}

fn newton_iteration(iterations: usize, x0: &[f32], func: impl Fn(&[f32]) -> Vec<f32>, hess: impl Fn(&[f32]) -> Mat<f32>) -> Vec<f32> {
    let mut x = ColRef::from_slice(x0).to_owned();
    for _ in 0..iterations {
        let x_owned: Vec<f32> = x.iter().copied().collect();
        let f = ColRef::from_slice(&func(&x_owned)).to_owned();
        let h = hess(&x_owned);
        x -= h.full_piv_lu().solve(&f);
    }
    x.iter().map(|v| *v).collect::<Vec<_>>()
}


impl Atmosphere {
    pub fn new(width: u32, mass: f32, to_sun: Vector3<f32>, spin: Vector3<f32>) -> Self {
        let resolution = (width/2) as usize;
        let psi = to_sun.dot(spin);
        let temperatures = {
            let mut out = [0.; 6];
            for face_index in 0..6 {
                let normal = match face_index {
                    0 => Vector3::unit_x(),
                    1 => -Vector3::unit_x(),
                    2 => Vector3::unit_y(),
                    3 => -Vector3::unit_y(),
                    4 => Vector3::unit_z(),
                    5 => -Vector3::unit_z(),
                    _ => unreachable!()
                };
                let theta = (spin.dot(normal)).acos();
                out[face_index] = (theta - psi).cos();
            }
            out
        };
        let winds = Self::get_basic_winds(&temperatures);
        let humidity_interp = SphericalInterpolator::new(|_| 0., 1);
        let wind_interp = SphericalInterpolator::new(|pos| {
            let linf_norm = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
            let index = 
                if linf_norm == pos.x.abs() { if pos.x > 0. { 0 } else { 1 } }
                else if linf_norm == pos.y.abs() { if pos.y > 0. { 2 } else { 3 } }
                else if linf_norm == pos.z.abs() { if pos.z > 0. { 4 } else { 5 } }
                else {unreachable!()};
            winds[index]
        },  resolution);
        let temp_interp = SphericalInterpolator::new(|pos| {
            let linf_norm = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
            let index = 
                if linf_norm == pos.x.abs() { if pos.x > 0. { 0 } else { 1 } }
                else if linf_norm == pos.y.abs() { if pos.y > 0. { 2 } else { 3 } }
                else if linf_norm == pos.z.abs() { if pos.z > 0. { 4 } else { 5 } }
                else {unreachable!()};
            temperatures[index]
        },  resolution);
        Self {
            wind_interp,
            humidity_interp,
            temp_interp,
            resolution,
        }
    }
    
    // TODO
    pub fn get_ocean_volume(&self) -> f32 {
        1000.
    }

    /// Get the basic Hadley Cell flow
    pub fn get_basic_winds(temps: &[f32; 6]) -> [Vector3<f32>; 6] {
        let output = newton_iteration(
            4,
            &vec![0.; 18],
            |x| { // Function
                let mut output = vec![0.; 18];
                for edge_index in 0..6 {
                    // Get indices of the adjoining faces
                    let (i0, i1) = match edge_index {
                        0 => (4,0),
                        1 => (0,2),
                        2 => (2,5),
                        3 => (5,3),
                        4 => (3,1),
                        5 => (1,0),
                        _ => unreachable!(),
                    };
                    let normal = match edge_index {
                        0 => Vector3::new(1., 0., 1.),
                        1 => Vector3::new(1., 1., 0.),
                        2 => Vector3::new(0., 1., -1.),
                        3 => Vector3::new(0., -1., -1.),
                        4 => Vector3::new(-1., -1., 0.),
                        5 => Vector3::new(-1., 0., 1.),
                        _ => unreachable!(),
                    } * 0.707;
                    let u0 = Vector3::new(x[i0*3+0], x[i0*3+1], x[i0*3+2]);
                    let u1 = Vector3::new(x[i1*3+0], x[i1*3+1], x[i1*3+2]);
                    let udot = normal.dot((u1+u0)/2.);
                    let eqn = udot.powi(2) * (u1 - u0) - normal * (
                        (temps[i1] - temps[i0]) * udot - normal.dot(u1-u0)
                    );
                    output[edge_index*3+0] = eqn.x;
                    output[edge_index*3+1] = eqn.y;
                    output[edge_index*3+2] = eqn.z;
                }
                output
            },
            |x| { // Hessian
                let mut output = Mat::zeros(18,18);
                for edge_index in 0..6 {
                    // Get indices of the adjoining faces
                    let (i0, i1) = match edge_index {
                        0 => (4,0),
                        1 => (0,2),
                        2 => (2,5),
                        3 => (5,3),
                        4 => (3,1),
                        5 => (1,0),
                        _ => unreachable!(),
                    };
                    let normal = match edge_index {
                        0 => Vector3::new(1., 0., 1.),
                        1 => Vector3::new(1., 1., 0.),
                        2 => Vector3::new(0., 1., -1.),
                        3 => Vector3::new(0., -1., -1.),
                        4 => Vector3::new(-1., -1., 0.),
                        5 => Vector3::new(-1., 0., 1.),
                        _ => unreachable!(),
                    } * 0.707;
                    let mut u0 = Vector3::new(x[i0*3+0], x[i0*3+1], x[i0*3+2]);
                    let mut u1 = Vector3::new(x[i1*3+0], x[i1*3+1], x[i1*3+2]);
                    let udot = normal.dot((u1+u0)/2.);
                    let eqn0 = udot.powi(2) * (u1 - u0) - normal * (
                        (temps[i1] - temps[i0]) * udot - normal.dot(u1-u0)
                    );

                    let delta = 0.01;
                    for i in 0..3 {
                        u0[i] += delta;
                        let udot = normal.dot((u1+u0)/2.);
                        let eqn = udot.powi(2) * (u1 - u0) - normal * (
                            (temps[i1] - temps[i0]) * udot - normal.dot(u1-u0)
                        );
                        let grad = (eqn - eqn0) / delta;
                        u0[i] -= delta;
                        output[((edge_index*3+0), (i0*3+i))] = grad.x;
                        output[((edge_index*3+1), (i0*3+i))] = grad.y;
                        output[((edge_index*3+2), (i0*3+i))] = grad.z;
                    }
                    for i in 0..3 {
                        u1[i] += delta;
                        let udot = normal.dot((u1+u0)/2.);
                        let eqn = udot.powi(2) * (u1 - u0) - normal * (
                            (temps[i1] - temps[i0]) * udot - normal.dot(u1-u0)
                        );
                        let grad = (eqn - eqn0) / delta;
                        u1[i] -= delta;
                        output[((edge_index*3+0), (i1*3+i))] = grad.x;
                        output[((edge_index*3+1), (i1*3+i))] = grad.y;
                        output[((edge_index*3+2), (i1*3+i))] = grad.z;
                    }
                }
                output
            }
        );
        [
            Vector3::new(output[0], output[1], output[2]),
            Vector3::new(output[3], output[4], output[5]),
            Vector3::new(output[6], output[7], output[8]),
            Vector3::new(output[9], output[10], output[11]),
            Vector3::new(output[12], output[13], output[14]),
            Vector3::new(output[15], output[16], output[17]),
        ]
    }

    /// Set the wind flow, humidity, and temperature interpolators given the planet terrain
    pub fn set_flow(&mut self, terrain: &Terrain) {
        let terrain_interp = terrain.get_interpolator(self.resolution);
        self.get_wind_interp(&terrain_interp);
        if let Some(alt) = terrain.ocean_alt {
            self.get_humidity_interp(alt, &terrain_interp);
        }
        self.get_temp_interp();
    }

    /// Compute the wind as a function of position using mountains
    fn get_wind_interp(&mut self, terrain_interp: &SphericalInterpolator<f32>) {
        self.wind_interp = SphericalInterpolator::new(|pos| {
            let local_wind = self.wind_interp.get(pos);
            let local_alt = terrain_interp.get(pos);

            let up = pos.normalize();
            let fore = Vector3::new(1., 2., 3.3).normalize();
            let left = up.cross(fore).normalize();
            let dx = 0.02;
            let local_grad = 
                fore * (terrain_interp.get(pos+fore*dx) - local_alt) / dx
                + left * (terrain_interp.get(pos+left*dx) - local_alt) / dx
            ; // OPTIMIZE interpolate the gradient
            let projection = local_grad * local_wind.dot(local_grad) / local_grad.magnitude2();
            let frac_to_remove = 1. - (-local_alt/MOUNTAIN_SCALE).exp();
            local_wind - projection * frac_to_remove
        }, self.resolution);
    }

    /// Compute the humidity as a function of position by back-integrating ocean coverage
    fn get_humidity_interp(&mut self, ocean_alt: f32, terrain_interp: &SphericalInterpolator<f32>) {
        let dt = 1.;
        let dsens_dt = 0.1;
        self.humidity_interp = SphericalInterpolator::new(|mut pos| {
            let mut sensitivity = 1.;
            for _ in 0..(3.*dsens_dt/dt) as usize {
                pos -= self.wind_interp.get(pos) * dt;
                sensitivity *= dsens_dt * dt;
                if terrain_interp.get(pos) < ocean_alt {
                    break;
                }
            }
            sensitivity
        }, self.resolution);
    }

    /// Compute the temperature as a function of position by back-integrating wind
    fn get_temp_interp(&mut self) {
        let dt = 1.;
        let dsens_dt = 0.1;
        self.temp_interp = SphericalInterpolator::new(|mut pos| {
            let mut temp = 0.;
            let mut sensitivity = 1./dsens_dt;
            for _ in 0..(3.*dsens_dt/dt) as usize {
                pos -= self.wind_interp.get(pos) * dt;
                sensitivity *= dsens_dt * dt;
                temp += (self.temp_interp.get(pos)) * sensitivity * dt;
                // TODO make the ocean cooler, mountains, etc. Adjust for humidity
            }
            temp
        }, self.resolution);
    }
}