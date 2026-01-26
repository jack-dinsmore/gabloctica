use cgmath::{InnerSpace, Matrix3, Quaternion, SquareMatrix, Vector3, Zero};

use crate::physics::{Physics, collisions::Collider};

#[derive(Clone, Debug)]
pub enum MoI {
    Diagonal{ moi: Vector3<f64>, reciprocals: Vector3<f64> },
    Matrix{ moi: Matrix3<f64>, inverse: Matrix3<f64>},
}
impl MoI {
    pub fn new_diagonal(moi: Vector3<f64>) -> Self {
        let reciprocals = Vector3::new(
            (moi.y - moi.z) / moi.x,
            (moi.z - moi.x) / moi.y,
            (moi.x - moi.y) / moi.z,
        );
        Self::Diagonal { moi, reciprocals }
    }
    pub fn new_matrix(moi: Matrix3<f64>) -> Self {
        Self::Matrix { inverse: moi.invert().unwrap(),  moi }
    }
    /// Get the angular acceleration for angular velocity `omega`.
    pub fn get_self_accel(&self, omega: Vector3<f64>) -> Vector3<f64> {
        match self {
            MoI::Diagonal { reciprocals, .. } => Vector3::new(
                omega.y*omega.z * reciprocals.x,
                omega.z*omega.x * reciprocals.y,
                omega.x*omega.y * reciprocals.z,
            ),
            MoI::Matrix { moi, inverse } => {
                inverse * omega.cross(moi * omega)
            },
        }
    }
    /// Multiply a vector by the MOI
    pub fn mul(&self, v: Vector3<f64>) -> Vector3<f64> {
        match self {
            MoI::Diagonal { moi, .. } => Vector3::new(
                v.x * moi.x,
                v.y * moi.y,
                v.z * moi.z,
            ),
            MoI::Matrix { moi, .. } => {
                moi * v
            },
        }
    }
    /// Multiply a vector by the MOI inverse
    pub fn mul_inv(&self, v: Vector3<f64>) -> Vector3<f64> {
        match self {
            MoI::Diagonal { moi, .. } => Vector3::new(
                v.x / moi.x,
                v.y / moi.y,
                v.z / moi.z,
            ),
            MoI::Matrix { inverse, .. } => {
                inverse * v
            },
        }
    }
    
    pub(crate) fn get_inv(&self) -> Matrix3<f64> {
        match self {
            MoI::Diagonal { moi, .. } => Matrix3::new(
                1. / moi.x,0.,0.,
                0.,1. / moi.y,0.,
                0.,0.,1. / moi.z,
            ),
            MoI::Matrix { inverse, .. } => {
                inverse.clone()
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RigidBody {
    physics: *mut Physics,
    pub(super) index: usize,
}
impl std::ops::Deref for RigidBody {
    type Target = RigidBodyData;

    fn deref(&self) -> &Self::Target {
        unsafe {&(&*self.physics).bodies[self.index]}
    }
}
impl std::ops::DerefMut for RigidBody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut (&mut *self.physics).bodies[self.index]}
    }
}
impl PartialEq for RigidBody {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl Eq for RigidBody {}
impl PartialOrd for RigidBody {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl RigidBody {
    pub fn new(physics: &mut Physics, init: RigidBodyInit) -> Self {
        let index = physics.bodies.len();

        // Initialize data
        let data = RigidBodyData {
            pos: init.pos,
            com_pos: init.com_pos,
            vel: init.vel,
            ori: init.ori,
            ang_vel: init.ang_vel,
            mass: init.mass,
            moi: init.moi,
            static_coeff: init.static_coeff,
            kinetic_coeff: init.kinetic_coeff,
            collider: init.collider,
            restitution: init.restitution,
            
            forces: Vector3::zero(),
            torques: Vector3::zero(),
        };
        if data.static_coeff < data.kinetic_coeff {
            panic!("Coefficient of static friction was < kinetic friction. This is probably a mistake");
        }
        if data.static_coeff < 0. || data.kinetic_coeff < 0. {
            panic!("Coefficients of friction must be non-negative");
        }
        if data.mass < 0. {
            panic!("Mass must be non-negative");
        }
        physics.bodies.push(data);

        // Create the RigidBody
        Self {
            physics: physics as *mut _,
            index,
        }
    }

    pub(super) fn from_index(physics: &mut Physics, index: usize) -> Self {
        Self {
            physics: physics as *mut _,
            index,
        }
    }
    
    pub fn get_object_collider_mut(&mut self) -> &mut super::collisions::shapes::ObjectData {
        if let Some(c) = &mut self.collider {
            if let Collider::Object(d) = c {
                return d;
            }
        }
        panic!("The collider was not type Object.")
    }
}

pub struct RigidBodyInit {
    pub pos: Vector3<f64>,
    pub com_pos: Vector3<f64>,
    pub vel: Vector3<f64>,
    /// Orientation rotates FROM body TO inertial
    pub ori: Quaternion<f64>,
    pub ang_vel: Vector3<f64>,
    pub mass: f64,
    pub moi: MoI,
    pub static_coeff: f64,
    pub kinetic_coeff: f64,
    pub collider: Option<Collider>,
    pub restitution: f64,
}
impl Default for RigidBodyInit {
    fn default() -> Self {
        Self {
            pos: Vector3::zero(),
            com_pos: Vector3::zero(),
            vel: Vector3::zero(),
            ori: Quaternion::new(0., 0., 0., 1.),
            ang_vel: Vector3::zero(),
            mass: 1.,
            static_coeff: 0.5,
            kinetic_coeff: 0.3,
            moi: MoI::new_diagonal(Vector3::new(1., 1., 1.)),
            collider: None,
            restitution: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
/// A point at position x on the RB is at inertial pos `pos + ori*(x-com_pos)`
pub struct RigidBodyData {
    /// Points to the center of mass
    pub pos: Vector3<f64>,
    /// Points from the body origin to the center of mass
    pub com_pos: Vector3<f64>,
    pub vel: Vector3<f64>,
    /// Rotates from the body to global
    pub ori: Quaternion<f64>,
    pub ang_vel: Vector3<f64>,
    pub mass: f64,
    pub moi: MoI,
    pub static_coeff: f64,
    pub kinetic_coeff: f64,
    pub collider: Option<Collider>,
    pub restitution: f64,

    pub(super) forces: Vector3<f64>,
    pub(super) torques: Vector3<f64>,
}
impl RigidBodyData {
    pub fn add_force(&mut self, force: Vector3<f64>) {
        self.forces += force;
    }
    pub fn add_torque(&mut self, torque: Vector3<f64>) {
        self.torques += torque;
    }
    pub fn add_couple(&mut self, force: Vector3<f64>, offset: Vector3<f64>) {
        self.add_force(force);
        self.add_torque(offset.cross(force));
    }
    pub(super) fn update(&mut self, delta_t: f64) {
        let rotated_torques = self.ori * self.torques;
        self.ang_vel += self.moi.mul_inv(rotated_torques) * delta_t;
        let vel_quat = Quaternion::from_sv(0., self.ang_vel);
        self.ori += 0.5 * vel_quat * self.ori * delta_t;
        self.ori.normalize();

        self.vel += self.forces * delta_t / self.mass;
        self.pos += self.vel * delta_t;

        self.forces = Vector3::zero();
        self.torques = Vector3::zero();
    }
}