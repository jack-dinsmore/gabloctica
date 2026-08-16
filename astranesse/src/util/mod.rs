mod interp;
mod vendor;
mod grid;

use std::{cell::RefCell, rc::Rc};
use cgmath::{Matrix3, Vector3};
pub use vendor::{Vendor, Tagged};
pub use interp::SphericalInterpolator;
pub use grid::{Grid, CHUNK_SIZE};
pub type RcCell<T> = Rc<RefCell<T>>;

pub fn my_fmod(f: f64, l: f64) -> f64 {
    let phase = f / l;
    (phase - phase.floor()) * l
}

pub fn outer(v1: Vector3<f64>, v2: Vector3<f64>) -> Matrix3<f64> {
    Matrix3::new(
        v1.x*v2.x,
        v1.x*v2.y,
        v1.x*v2.z,
        v1.y*v2.x,
        v1.y*v2.y,
        v1.y*v2.z,
        v1.z*v2.x,
        v1.z*v2.y,
        v1.z*v2.z,
    )
}