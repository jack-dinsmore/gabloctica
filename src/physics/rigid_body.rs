use cgmath::{InnerSpace, Quaternion, Vector3, Zero};

use crate::physics::{Physics, collisions::Collider};

#[derive(Debug, Clone, Copy)]
pub struct RigidBody {
    physics: *mut Physics,
    index: usize,
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
        let moi_div = Vector3::new(
            (init.moi.y - init.moi.z) / init.moi.x,
            (init.moi.z - init.moi.x) / init.moi.y,
            (init.moi.x - init.moi.y) / init.moi.z,
        );
        let data = RigidBodyData {
            pos: init.pos,
            vel: init.vel,
            ori: init.ori,
            ang_vel: init.ang_vel,
            mass: init.mass,
            moi: init.moi,
            static_coeff: init.static_coeff,
            kinetic_coeff: init.kinetic_coeff,
            collider: init.collider,
            
            moi_div,
            contacts: Vec::new(),
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
        if data.moi.x < 0. || data.moi.y < 0. || data.moi.z < 0. {
            panic!("MoI must be non-negative");
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
    pub vel: Vector3<f64>,
    /// Orientation rotates FROM body TO inertial
    pub ori: Quaternion<f64>,
    pub ang_vel: Vector3<f64>,
    pub mass: f64,
    pub moi: Vector3<f64>,
    pub static_coeff: f64,
    pub kinetic_coeff: f64,
    pub collider: Option<Collider>,
}
impl Default for RigidBodyInit {
    fn default() -> Self {
        Self {
            pos: Vector3::zero(),
            vel: Vector3::zero(),
            ori: Quaternion::new(0., 0., 0., 1.),
            ang_vel: Vector3::zero(),
            mass: 1.,
            static_coeff: 0.5,
            kinetic_coeff: 0.3,
            moi: Vector3::new(1., 1., 1.),
            collider: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RigidBodyData {
    pub pos: Vector3<f64>,
    pub vel: Vector3<f64>,
    pub ori: Quaternion<f64>,
    pub ang_vel: Vector3<f64>,
    pub mass: f64,
    moi: Vector3<f64>,
    pub static_coeff: f64,
    pub kinetic_coeff: f64,
    pub collider: Option<Collider>,

    moi_div: Vector3<f64>,
    contacts: Vec<RigidBody>,
    forces: Vector3<f64>,
    torques: Vector3<f64>,
}
impl RigidBodyData {
    pub fn add_force(&mut self, force: Vector3<f64>) {
        // TODO transmit forces and torques when in contact with something
        self.forces += force;
    }
    pub fn add_torque(&mut self, torque: Vector3<f64>) {
        // TODO transmit forces and torques when in contact with something
        self.torques += torque;
    }
    pub fn add_couple(&mut self, force: Vector3<f64>, offset: Vector3<f64>) {
        self.add_force(force);
        self.add_torque(offset.cross(force));
    }
    pub fn set_moi(&mut self, moi: Vector3<f64>) {
        self.moi = moi;
        self.moi_div = Vector3::new(
            (moi.y - moi.z) / moi.x,
            (moi.z - moi.x) / moi.y,
            (moi.x - moi.y) / moi.z,
        );
    }
    pub(super) fn update(&mut self, delta_t: f64) {
        let rotated_torques = self.ori * self.torques;
        self.ang_vel += Vector3::new(
            rotated_torques.x / self.moi.x + self.ang_vel.y*self.ang_vel.z * self.moi_div.x,
            rotated_torques.y / self.moi.y + self.ang_vel.z*self.ang_vel.x * self.moi_div.y,
            rotated_torques.z / self.moi.z + self.ang_vel.x*self.ang_vel.y * self.moi_div.z,
        ) * delta_t;
        let vel_quat = Quaternion::from_sv(0., self.ang_vel);
        self.ori += 0.5 * vel_quat * self.ori * delta_t;
        self.ori.normalize();

        self.vel += self.forces * delta_t / self.mass;
        self.pos += self.vel * delta_t;

        self.forces = Vector3::zero();
        self.torques = Vector3::zero();
    }
}