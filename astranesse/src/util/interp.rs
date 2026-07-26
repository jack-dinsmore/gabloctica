use cgmath::Vector3;
use std::ops::{Mul, Add};

#[derive(Clone, Debug)]
pub struct SphericalInterpolator<T: Mul<f32> + Add<T> + Clone> {
    data: Vec<T>,
    resolution: f32,
    span: usize,
    delta: f32,
}

impl<T: Mul<f32, Output=T> + Add<T, Output=T> + Clone> SphericalInterpolator<T> {
    pub fn new(f: impl Fn(Vector3<f32>)->T, resolution: usize) -> Self {
        let span = resolution+1;
        let mut data = Vec::with_capacity(6*span*span);
        for face_index in 0..6 {
            for u in 0..=resolution {
                let ux = u as f32 / resolution as f32;
                for v in 0..=resolution {
                    let vx = v as f32 / resolution as f32;
                    let mut pos = match face_index {
                        0 => Vector3::new(1., ux, vx),
                        1 => Vector3::new(-1., ux, vx),
                        2 => Vector3::new(ux, 1., vx),
                        3 => Vector3::new(ux, -1., vx),
                        4 => Vector3::new(ux, vx, 1.),
                        5 => Vector3::new(ux, vx, -1.),
                        _ => unreachable!()
                    };
                    pos *= 2.;
                    pos -= Vector3::new(1., 1., 1.);
                    data.push(f(pos));
                }
            }
        }

        Self {
            data,
            resolution: resolution as f32,
            span,
            delta: 1./resolution as f32
        }
    }

    pub fn get(&self, mut pos: Vector3<f32>) -> T {
        let max = pos.x.abs().max(pos.y.abs().max(pos.z));
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

        let u_index = (u * self.resolution) as usize;
        let v_index = (v * self.resolution) as usize;
        let offset = self.span*self.span*face_index;
        let q0 = self.data[offset + u_index*self.span + v_index].clone();
        let q1 = self.data[offset + (u_index+1)*self.span + v_index].clone();
        let q2 = self.data[offset + u_index*self.span + v_index+1].clone();
        let q3 = self.data[offset + (u_index+1)*self.span + v_index+1].clone();
        let dx = u * self.resolution - u_index as f32;
        let dy = v * self.resolution - v_index as f32;
        
        q0*(1.-dx)*(1.-dy) + q1*dx*(1.-dy) + q2*(1.-dx)*dy + q3*dx*dy
    }
}