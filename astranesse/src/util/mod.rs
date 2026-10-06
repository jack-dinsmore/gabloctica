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

pub fn moi_product(v: Vector3<f64>) -> Matrix3<f64> {
    Matrix3::new(
        v.y*v.y + v.z*v.z, -v.x*v.y, -v.x*v.z,
        -v.y*v.x, v.x*v.x + v.z*v.z, -v.y*v.z,
        -v.x*v.z, -v.z*v.y, v.x*v.x + v.y*v.y,
    )
}