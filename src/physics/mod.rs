mod rigid_body;
mod collisions;

pub use rigid_body::{RigidBody, RigidBodyInit};
pub use collisions::{Collider, CollisionReport};

use cgmath::InnerSpace;
use crate::physics::rigid_body::RigidBodyData;


const NEWTON_G: f64 = 1.;
const GRAVITY_THRESH: f64 = 1e6;

pub struct Physics {
    bodies: Vec<RigidBodyData>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        // Gravity
        for i in 0..self.bodies.len() {
            let planet = RigidBody::from_index(self, i);
            if planet.mass < GRAVITY_THRESH {continue;}
            for j in 0..self.bodies.len() {
                if j == i {continue;}
                let mut part = RigidBody::from_index(self, j);
                if part.mass > GRAVITY_THRESH {continue;}

                let dist = part.pos - planet.pos;
                let force = -NEWTON_G * planet.mass * part.mass * dist / dist.magnitude().powi(3);
                part.add_force(force);
            }
        }

        // Collisions
        let physics = self as *mut _;
        for i in 0..self.bodies.len() {
            let a = RigidBody::from_index(self, i);
            for j in (i+1)..self.bodies.len() {
                let b = RigidBody::from_index(self, j);
                if let CollisionReport::Some { normal, depth, p1, p2 } = Collider::check_collision(&a, &b) {
                    // handle collisions
                    unimplemented!()
                }
            }
        }

        // Force updates
        for body in &mut self.bodies {
            body.update(delta_t);
        }
    }
}