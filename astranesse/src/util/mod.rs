mod interp;
mod vendor;
mod grid;
mod kmeans;
mod orientations;
mod colors;

use std::{cell::RefCell, rc::Rc};
use cgmath::{Matrix3, Vector3};
pub use vendor::{Vendor, Tagged};
pub use interp::{SphericalInterpolator, lerp};
pub use kmeans::{KMeans};
pub use grid::{Grid, CHUNK_SIZE};
pub use orientations::Ori;
pub use colors::*;
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

pub fn get_face_index(pos: Vector3<f32>) -> u8 {
    let max = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
    if max == pos.x.abs() {
        if pos.x > 0. { 0 } else { 1 }
    } else if max == pos.y.abs() {
        if pos.y > 0. { 2 } else { 3 }
    } else {
        if pos.z > 0. { 4 } else { 5 }
    }
}