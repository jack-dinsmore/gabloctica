mod interp;

use std::{cell::RefCell, rc::Rc};

pub use interp::SphericalInterpolator;
pub type RcCell<T> = Rc<RefCell<T>>;