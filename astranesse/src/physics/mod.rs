mod rigid_body;
mod collisions;

use faer::{Mat, Side, prelude::Solve};
use cgmath::{InnerSpace, Matrix3, Vector3};
pub use rigid_body::{RigidBodyData, RigidBody, RigidBodyInit, MoI};
pub use collisions::{shapes, Collider, CollisionReport};
use crate::util::Vendor;

pub const NEWTON_G: f64 = 5.;
const GRAVITY_THRESH: f64 = 10.;

pub struct Physics {
    collision_pairs: Vec<(RigidBody, RigidBody)>,
    collision_reports: Vec<CollisionReport>,
    pub rb_vendor: Vendor<RigidBodyData>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            rb_vendor: Vendor::new(),
            collision_pairs: Vec::new(),
            collision_reports: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        // Gravity
        for (i, planet) in self.rb_vendor.iter().enumerate() {
            if planet.mass < GRAVITY_THRESH {continue;}
            for (j, mut part) in &mut self.rb_vendor.iter().enumerate() {
                if j == i {continue;}
                if part.mass > GRAVITY_THRESH {continue;}
                
                let dist = part.pos - planet.pos;
                let force = -NEWTON_G * planet.mass * part.mass * dist / dist.magnitude().powi(3);
                part.add_force(force);
            }
        }

        // Friction
        for ((a, b), report) in self.collision_pairs.iter_mut().zip(&self.collision_reports) {
            let relative_force = a.forces - b.forces;
            match report {
                CollisionReport::Some { normal, depth, p1, p2 } => {
                    let rel_vel = a.vel - b.vel; // TODO rotation
                    let force = if rel_vel.magnitude() < 0.001 {
                        // Static
                        let coeff = (a.static_coeff + b.static_coeff) / 2.;
                        let mut force_dir = a.forces - b.forces;
                        force_dir -= force_dir.dot(*normal) * force_dir;
                        force_dir = force_dir.normalize();
                        force_dir * normal.dot(relative_force) * coeff
                    } else {
                        // Kinetic
                        let coeff = (a.kinetic_coeff + b.kinetic_coeff) / 2.;
                        let mut force_dir = rel_vel;
                        force_dir -= force_dir.dot(*normal) * force_dir;
                        force_dir = force_dir.normalize();
                        force_dir * normal.dot(relative_force) * coeff
                    };
                    a.add_force(force);
                    b.add_force(force);
                    a.add_torque(p1.cross(force));
                    b.add_torque(p2.cross(force));
                },
                CollisionReport::None => unreachable!(),
            }
        }

        // Collisions
        self.collision_pairs.clear();
        self.collision_reports.clear();
        for (i, mut a) in self.rb_vendor.iter().enumerate() {
            for (j, mut  b) in self.rb_vendor.iter().enumerate() {
                if j >= i {break;}
                let report = Collider::check_collision(&a, &b);
                if let CollisionReport::Some { normal, depth, p1, p2 } = report {
                    // handle collisions
                    let denom = (1. / a.mass + 1. / b.mass) + 
                        a.moi.mul_inv(p1.cross(normal)).cross(p1).dot(normal) + 
                        b.moi.mul_inv(p2.cross(normal)).cross(p2).dot(normal)
                    ;
                    let rel_vel = a.vel - b.vel;
                    let restitution = (a.restitution + b.restitution) / 2.;
                    let impulse = (1. + restitution) * rel_vel.dot(normal) / denom;
                    a.add_force(-impulse / delta_t * normal);
                    b.add_force(impulse / delta_t * normal);

                    let a_mass_frac = a.mass / (a.mass + b.mass);
                    a.pos -= normal*depth * (1. - a_mass_frac);
                    b.pos += normal*depth * a_mass_frac;

                    self.collision_pairs.push((a.clone(), b.clone()));
                    self.collision_reports.push(report);
                }
            }
        }

        // Compute self angular acceleration
        for mut body in self.rb_vendor.iter() {
            let accel = body.moi.get_self_accel(body.ang_vel);
            body.add_torque(accel);
        }
        // self.resolve_normal_forces(); // TODO
        // self.resolve_normal_torques();

        // Force updates
        for mut body in self.rb_vendor.iter() {
            body.update(delta_t);
        }
    }

    pub fn resolve_normal_forces(&mut self) {
        if self.collision_reports.is_empty() {return;}
        let dimension = self.collision_reports.len();
        let mut m: Mat<f64> = Mat::zeros(dimension, dimension);
        let mut k = Mat::zeros(dimension, 1);
        for i in 0..dimension {
            let a = self.collision_pairs[i].0.clone();
            let b = self.collision_pairs[i].1.clone();
            let normal = match self.collision_reports[i] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, .. } => normal,
            };
            k[(i, 0)] = (a.forces / a.mass - b.forces / b.mass).dot(normal);
            m[(i,i)] += 1./a.mass + 1./b.mass;
            for j in (i+1)..dimension {
                let c = self.collision_pairs[j].0.clone();
                let d = self.collision_pairs[j].1.clone();
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
            let mut normal = match self.collision_reports[i] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, .. } => normal,
            };
            normal *= k[(i, 0)];
            a.add_force(normal);
            b.add_force(-normal);
        }
    }

    pub fn resolve_normal_torques(&mut self) {
        let n_pairs = self.collision_reports.len();
        if self.collision_reports.is_empty() {return;}
        let dimension = n_pairs*3;
        let mut m: Mat<f64> = Mat::zeros(dimension, dimension);
        let mut k = Mat::zeros(dimension,  1);
        let mut write_block = |i: usize, j: usize, mat: Matrix3<f64>| {
            for k in 0..3 {
                for l in 0..3 {
                    m[(3*i+l, 3*j+k)] = mat[l][k];
                }
            }
        };

        for i in 0..n_pairs {
            let a = self.collision_pairs[i].0.clone();
            let b = self.collision_pairs[i].1.clone();
            let vec = a.moi.mul_inv(a.torques) - b.moi.mul_inv(b.torques / b.mass);
            k[(3*i+0, 0)] = vec.x;
            k[(3*i+1, 0)] = vec.y;
            k[(3*i+2, 0)] = vec.z;

            let a_inv = a.moi.get_inv();
            let b_inv = b.moi.get_inv();
            write_block(i, i, a_inv + b_inv);
            for j in (i+1)..n_pairs {
                let c = self.collision_pairs[j].0.clone();
                let d = self.collision_pairs[j].1.clone();
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
            let normal = match self.collision_reports[i] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, .. } => normal,
            };
            let mut accel = Vector3::new(
                k[(3*i+0, 0)],
                k[(3*i+1, 0)],
                k[(3*i+2, 0)],
            );
            accel -= normal * normal.dot(accel);
            a.add_torque(accel);
            b.add_torque(-accel);
        }
    }
}