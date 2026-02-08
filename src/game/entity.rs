use cgmath::{Quaternion, Vector3, Zero};

use crate::physics::{Physics, RigidBody, RigidBodyInit, shapes::BoxData};

pub struct Entity {
    pub body: RigidBody
}

impl Entity {
    pub fn new(physics: &mut Physics, mut init: RigidBodyInit) -> Self {
        init.collider = Some(crate::physics::Collider::Box(BoxData { 
            center: Vector3::zero(),
            edges: Vector3::new(0.9, 0.4, 0.4),
        }));
        init.moi = crate::physics::MoI::new_diagonal(Vector3::new(1e8, 1e8, 1e8));
        Self {
            body: RigidBody::new(physics, init),
        }
    }

    pub fn walk(&mut self, dir: Vector3<f64>) {
        self.body.add_force(dir);
    }

    pub fn update(&mut self, delta_t: f64) {
        self.body.ori = Quaternion::new(1., 0., 0., 0.);
    }
}