mod interp;
mod vendor;
mod grid;

use std::{cell::RefCell, rc::Rc};
use cgmath::{Matrix3, Vector3};
pub use vendor::{Vendor, Tagged};
pub use interp::{SphericalInterpolator, lerp};
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

/// Find the pth percentile of a slice `data` assuming the slice is sorted.
pub fn percentile_sorted(data: &[f32], p: f32) -> f32 {
    let rank = p / 100.0 * (data.len() - 1) as f32;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let fraction = rank - lower as f32;

    data[lower] + fraction * (data[upper] - data[lower])
}