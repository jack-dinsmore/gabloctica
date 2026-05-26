mod interp;
pub mod vendor;
pub use vendor::{Vendor, Tagged};

use std::{cell::RefCell, rc::Rc};

pub use interp::SphericalInterpolator;
pub type RcCell<T> = Rc<RefCell<T>>;

pub fn my_fmod(f: f64, l: f64) -> f64 {
    let phase = f / l;
    (phase - phase.floor()) * l
}