use cgmath::{InnerSpace, Vector3};
use std::ops::{Mul, Add};

pub fn lerp(x: f32, x0: f32, x1: f32, y0: f32, y1: f32) -> f32{
    let frac = (x - x0) / (x1 - x0);
    frac * y1 + (1. - frac) * y0
}

#[derive(Clone, Debug)]
pub struct SphericalInterpolator<T> {
    data: Vec<T>,
    resolution: usize,
    span: usize,
    delta: f32,
}

impl<T> SphericalInterpolator<T> {
    /// Get the position of a point in the data vector. Returns an unnormalized vector
    fn get_pos(resolution: usize, face_index: usize, u: usize, v: usize) -> Vector3<f32> {
        let ux = u as f32 / resolution as f32;
        let vx = v as f32 / resolution as f32;
        let mut pos = match face_index {
            0 => Vector3::new(1., ux, vx),
            1 => Vector3::new(0., ux, vx),
            2 => Vector3::new(ux, 1., vx),
            3 => Vector3::new(ux, 0., vx),
            4 => Vector3::new(ux, vx, 1.),
            5 => Vector3::new(ux, vx, 0.),
            _ => unreachable!()
        };
        pos *= 2.;
        pos -= Vector3::new(1., 1., 1.);
        pos
    }

    /// Get the three unit axes at a point, with the first pointing out of plane
    fn get_axes(pos: Vector3<f32>) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let pos = pos.normalize();
        let mut ax1 = pos.cross(Vector3::unit_z());
        let mut mag2 = ax1.magnitude2();
        if mag2 == 0. {
            ax1 = pos.cross(Vector3::unit_x());
            mag2 = ax1.magnitude2();
        }
        ax1 /= mag2.sqrt();

        let ax2 = pos.cross(ax1);

        (pos, ax1, ax2)
    }
}

impl<T: Mul<f32, Output=T> + Add<T, Output=T> + Clone> SphericalInterpolator<T> {
    pub fn new(f: impl Fn(Vector3<f32>)->T, resolution: usize) -> Self {
        let span = resolution+1;
        let mut data = Vec::with_capacity(6*span*span);
        for face_index in 0..6 {
            for u in 0..=resolution {
                for v in 0..=resolution {
                    data.push(f(Self::get_pos(resolution, face_index, u, v)));
                }
            }
        }

        Self {
            data,
            resolution,
            span,
            delta: 1./resolution as f32
        }
    }

    pub fn get(&self, mut pos: Vector3<f32>) -> T {
        let max = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
        let face_index = if max == pos.x.abs() {
            if pos.x > 0. { 0 } else { 1 }
        } else if max == pos.y.abs() {
            if pos.y > 0. { 2 } else { 3 }
        } else {
            if pos.z > 0. { 4 } else { 5 }
        };
        pos /= max; // Lies in cube with corners (-1,-1,-1), (1,1,1)
        pos = pos/2. + Vector3::new(0.5, 0.5, 0.5); // Lies in cube with corners (0,0,0), (1,1,1)

        let (u, v) = match face_index {
            0|1 => (pos.y, pos.z),
            2|3 => (pos.x, pos.z),
            4|5 => (pos.x, pos.y),
            _ => unreachable!()
        };

        let u_index = (u * self.resolution as f32) as usize;
        let v_index = (v * self.resolution as f32) as usize;
        let offset = self.span*self.span*face_index;
        let q0 = self.data[offset + u_index*self.span + v_index].clone();
        let q1 = self.data[offset + (u_index+1)*self.span + v_index].clone();
        let q2 = self.data[offset + u_index*self.span + v_index+1].clone();
        let q3 = self.data[offset + (u_index+1)*self.span + v_index+1].clone();
        let dx = u * self.resolution as f32 - u_index as f32;
        let dy = v * self.resolution as f32 - v_index as f32;
        
        q0*(1.-dx)*(1.-dy) + q1*dx*(1.-dy) + q2*(1.-dx)*dy + q3*dx*dy
    }

    pub fn get_nearest(&mut self, mut pos: Vector3<f32>) -> &mut T {
        let max = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
        let face_index = if max == pos.x.abs() {
            if pos.x > 0. { 0 } else { 1 }
        } else if max == pos.y.abs() {
            if pos.y > 0. { 2 } else { 3 }
        } else {
            if pos.z > 0. { 4 } else { 5 }
        };
        pos /= max; // Lies in cube with corners (-1,-1,-1), (1,1,1)
        pos = pos/2. + Vector3::new(0.5, 0.5, 0.5); // Lies in cube with corners (0,0,0), (1,1,1)

        let (u, v) = match face_index {
            0|1 => (pos.y, pos.z),
            2|3 => (pos.x, pos.z),
            4|5 => (pos.x, pos.y),
            _ => unreachable!()
        };

        let u_index = (u * self.resolution as f32) as usize;
        let v_index = (v * self.resolution as f32) as usize;
        let offset = self.span*self.span*face_index;
        let dx = u * self.resolution as f32 - u_index as f32;
        let dy = v * self.resolution as f32 - v_index as f32;
        if dx < 0.5 && dy < 0.5 {
            &mut self.data[offset + u_index*self.span + v_index]
        } else if dx > 0.5 && dy < 0.5 {
            &mut self.data[offset + (u_index+1)*self.span + v_index]
        } else if dx < 0.5 && dy > 0.5 {
            &mut self.data[offset + u_index*self.span + v_index+1]
        } else {
            &mut self.data[offset + (u_index+1)*self.span + v_index+1]
        }
    }
}

impl SphericalInterpolator<Vector3<f32>> {
    pub fn get_divergence(&self, pos: Vector3<f32>) -> f32 {
        let step = self.delta/2.;
        let (pos, ax1, ax2) = Self::get_axes(pos);
        let forward = self.get(pos+ax1*step);
        let backward = self.get(pos-ax2*step);
        let left = self.get(pos+ax2*step);
        let right = self.get(pos-ax2*step);
        ((forward - backward).dot(ax1) + (left - right).dot(ax2))/(2.*step)
    }
}

impl SphericalInterpolator<f32> {
    // TODO
}