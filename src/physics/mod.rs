mod rigid_body;
mod collisions;

use std::{collections::BTreeSet, ops::DerefMut};

use faer::{Mat, Side, prelude::Solve};
pub use rigid_body::{RigidBody, RigidBodyInit, MoI};
pub use collisions::{Collider, CollisionReport};

use cgmath::{InnerSpace, Matrix3, Vector3};
use crate::physics::rigid_body::RigidBodyData;


pub const NEWTON_G: f64 = 1.;
const GRAVITY_THRESH: f64 = 1e6;

pub struct Physics {
    bodies: Vec<RigidBodyData>,
    collision_pairs: Vec<(RigidBody, RigidBody)>,
    collision_normals: Vec<Vector3<f64>>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            collision_pairs: Vec::new(),
            collision_normals: Vec::new(),
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
        self.collision_pairs.clear();
        self.collision_normals.clear();
        for i in 0..self.bodies.len() {
            let mut a = RigidBody::from_index(self, i);
            for j in (i+1)..self.bodies.len() {
                let mut b = RigidBody::from_index(self, j);
                if let CollisionReport::Some { normal, depth, p1, p2 } = Collider::check_collision(&a, &b) {
                    // handle collisions
                    let denom = (1. / a.mass + 1. / b.mass) + 
                        a.moi.mul_inv(p1.cross(normal)).cross(p1).dot(normal) + 
                        b.moi.mul_inv(p2.cross(normal)).cross(p2).dot(normal)
                    ;
                    let restitution = (a.restitution + b.restitution) / 2.;
                    let impulse = (1. + restitution) / denom;
                    a.add_force(impulse / delta_t * normal);
                    b.add_force(-impulse / delta_t * normal);

                    a.pos += normal*depth;
                    b.pos -= normal*depth;

                    self.collision_pairs.push((a, b));
                    self.collision_normals.push(normal);
                }
            }
        }

        // Compute self angular acceleration
        for body in &mut self.bodies {
            body.add_torque(body.moi.get_self_accel(body.ang_vel));
        }
        self.resolve_normal_forces();
        self.resolve_normal_torques();

        // Force updates
        for body in &mut self.bodies {
            body.update(delta_t);
        }
    }

    pub fn resolve_normal_forces(&mut self) {
        if self.collision_normals.is_empty() {return;}
        let dimension = self.collision_normals.len();
        let mut m: Mat<f64> = Mat::zeros(dimension, dimension);
        let mut k = Mat::zeros(1,dimension);
        for i in 0..dimension {
            let a = self.collision_pairs[i].0;
            let b = self.collision_pairs[i].1;
            let normal = self.collision_normals[i];
            k[(0,i)] = (a.forces / a.mass - b.forces / b.mass).dot(normal);
            m[(i,i)] += 1./a.mass + 1./b.mass;
            for j in (i+1)..dimension {
                let c = self.collision_pairs[j].0;
                let d = self.collision_pairs[j].1;
                if a == c {
                    m[(i,j)] += 1./a.mass;
                    m[(j,i)] += 1./a.mass;
                }
                if a == d {
                    m[(i,j)] += 1./a.mass;
                    m[(j,i)] += 1./a.mass;
                }
                if b == c {
                    m[(i,j)] += 1./b.mass;
                    m[(j,i)] += 1./b.mass;
                }
                if b == d {
                    m[(i,j)] += 1./b.mass;
                    m[(j,i)] += 1./b.mass;
                }
            }
        }
        m.llt(Side::Upper).unwrap().solve_in_place(&mut k);

        for (i, (a, b)) in self.collision_pairs.iter_mut().enumerate() {
            let normal = self.collision_normals[i] * k[(0,i)];
            a.add_force(normal);
            b.add_force(-normal);
        }
    }

    pub fn resolve_normal_torques(&mut self) {
        if self.collision_normals.is_empty() {return;}
        let dimension = self.collision_normals.len()*3;
        let mut m: Mat<f64> = Mat::zeros(dimension, dimension);
        let mut k = Mat::zeros(1,dimension);
        let mut write_block = |i: usize, j: usize, mat: Matrix3<f64>| {
            for k in 0..3 {
                for l in 0..3 {
                    m[(3*i+l, 3*j+k)] = mat[l][k];
                }
            }
        };

        for i in 0..dimension {
            let a = self.collision_pairs[i].0;
            let b = self.collision_pairs[i].1;
            let vec = a.moi.mul_inv(a.torques) - b.moi.mul_inv(b.torques / b.mass);
            k[(0, 3*i+0)] = vec.x;
            k[(0, 3*i+1)] = vec.y;
            k[(0, 3*i+2)] = vec.z;

            let a_inv = a.moi.get_inv();
            let b_inv = b.moi.get_inv();
            write_block(i, i, a_inv + b_inv);
            for j in (i+1)..dimension {
                let c = self.collision_pairs[j].0;
                let d = self.collision_pairs[j].1;
                if a == c {
                    write_block(i,j,a_inv);
                    write_block(j,i,a_inv);
                }
                if a == d {
                    write_block(i,j,a_inv);
                    write_block(j,i,a_inv);
                }
                if b == c {
                    write_block(i,j,b_inv);
                    write_block(j,i,b_inv);
                }
                if b == d {
                    write_block(i,j,b_inv);
                    write_block(j,i,b_inv);
                }
            }
        }
        m.llt(Side::Upper).unwrap().solve_in_place(&mut k);

        for (i, (a, b)) in self.collision_pairs.iter_mut().enumerate() {
            let normal = self.collision_normals[i];
            let mut accel = Vector3::new(
                k[(0, 3*i+0)],
                k[(0, 3*i+1)],
                k[(0, 3*i+2)],
            );
            accel -= normal * normal.dot(accel);
            a.add_torque(accel);
            b.add_torque(-accel);
        }
    }
}