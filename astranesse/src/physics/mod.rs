mod rigid_body;
mod collisions;

use faer::{Mat, Side, prelude::Solve};
use cgmath::{InnerSpace, Rotation, Vector3};
pub use rigid_body::{RigidBodyData, RigidBody, RigidBodyInit, MoI};
pub use collisions::{shapes, Collider, CollisionReport};
use rustc_hash::FxHashSet;
use crate::util::Vendor;

pub const NEWTON_G: f64 = 1e-2;
pub const STEFAN_BOLTZMANN: f64 = 1.;
pub const PROTON_MASS: f32 = 1.;
pub const BOLTZMANN: f32 = 5.;
const GRAVITY_P: i32 = 20;

fn gravity(x: Vector3<f64>, v: Vector3<f64>, mass: f64, warp_length: f64) -> Vector3<f64> {
    let r_big = x.magnitude();
    let e = x / r_big;
    let ae = Vector3::new(e.x.abs(), e.y.abs(), e.z.abs());
    let q = (e.x.abs().powi(GRAVITY_P) + e.y.abs().powi(GRAVITY_P) + e.z.abs().powi(GRAVITY_P)).powf(1.0 / GRAVITY_P as f64);
    let q_fac = q.powi(1 - GRAVITY_P);
    let grad_pe = Vector3::new(
        e.x.signum() * ae.x.powi(GRAVITY_P - 1) * q_fac,
        e.y.signum() * ae.y.powi(GRAVITY_P - 1) * q_fac,
        e.z.signum() * ae.z.powi(GRAVITY_P - 1) * q_fac,
    );
    let u = q - 1.0;

    let s = r_big + u * warp_length;
    let fv = r_big / s;

    let gamma = -(warp_length / (s * s)) * (grad_pe.dot(v) - (2.0 * q - 1.0) * e.dot(v));
    let vel = (v - e * (s * gamma)) / fv;

    let a = e.dot(vel);
    let b = vel.dot(vel);
    let sig = grad_pe.dot(vel);
    let weighted = ae.x.powi(GRAVITY_P - 2) * vel.x * vel.x
        + ae.y.powi(GRAVITY_P - 2) * vel.y * vel.y
        + ae.z.powi(GRAVITY_P - 2) * vel.z * vel.z;
    let phi = (GRAVITY_P - 1) as f64 / q * (q.powi(2 - GRAVITY_P) * weighted - sig * sig)
        - 4.0 * sig * a
        - q * b
        + 5.0 * q * a * a
        + u * (3.0 * a * a - b);

    -e * (mass * NEWTON_G / (s * s)) + (vel * (2.0 * gamma) - e * (warp_length * phi / (s * s)))
}

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

    pub fn contacting(&self, a: &RigidBody, b: &RigidBody) -> bool {
        for (i, j) in &self.collision_pairs {
            if (i == a && j == b) || (i == b && j == a) {
                return true;
            }
        }
        false
    }

    pub fn update(&mut self, delta_t: f64) {
        // Gravity
        for (i, planet) in self.rb_vendor.iter().enumerate() {
            if let Some(warp_length) = planet.warp_length {
                for (j, mut part) in &mut self.rb_vendor.iter().enumerate() {
                    if j == i {continue;}
                    if let None = part.warp_length {
                        let dpos = planet.ori.invert() * (part.pos - planet.pos);
                        let dvel = planet.ori.invert() * (part.vel - planet.vel);
                        let force = part.mass * gravity(dpos, dvel, planet.mass, warp_length);
                        part.add_force(planet.ori * force);
                    }
                }
            }
        }

        // Collisions
        self.collision_pairs.clear();
        self.collision_reports.clear();
        for (i, a) in self.rb_vendor.iter().enumerate() {
            for (j,  b) in self.rb_vendor.iter().enumerate() {
                if j >= i {break;}
                let report = Collider::check_collision(&a, &b);
                if report.is_some() {
                    self.collision_pairs.push((a.clone(), b.clone()));
                    self.collision_reports.push(report);
                }
            }
        }

        // Compute self angular acceleration
        for mut body in self.rb_vendor.iter() {
            let torque = body.moi.get_self_torque(body.ang_vel);
            body.add_torque(torque);
        }
        self.resolve_normal_forces(delta_t);

        // Force updates
        for mut body in self.rb_vendor.iter() {
            body.update(delta_t);
        }

        // Collision offsets
        for ((a, b), report) in self.collision_pairs.iter_mut().zip(&self.collision_reports) {
            if let CollisionReport::Some { normal, depth, p1, p2 } = report {
                let a_mass_frac = a.mass / (a.mass + b.mass);
                a.pos -= normal* *depth * (1. - a_mass_frac);
                b.pos += normal* *depth * a_mass_frac;
            }
        }
    }

    pub fn resolve_normal_forces(&mut self, delta_t: f64) {
        if self.collision_reports.is_empty() {return;}

        let mut collision_bodies: Vec<FxHashSet<RigidBody>> = Vec::new();
        let mut collision_indices: Vec<Vec<usize>> = Vec::new();
        for (k, (i, j)) in self.collision_pairs.iter().enumerate() {
            let (normal_k, pi, pj) = match self.collision_reports[k] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, p1, p2, .. } => (normal, p1, p2),
            };

            let mut found = false;
            for index in 0..collision_bodies.len() {
                if collision_bodies[index].contains(i) || collision_bodies[index].contains(j) {
                    collision_bodies[index].insert(i.clone());
                    collision_bodies[index].insert(j.clone());
                    collision_indices[index].push(k);
                    found = true;
                    break;
                }
            }
            if !found {
                let mut set = FxHashSet::default();
                set.insert(i.clone());
                set.insert(j.clone());
                collision_bodies.push(set);
                collision_indices.push(vec![k]);
            }
        }
        for indices in collision_indices {
            self.resolve_normal_forces_chunk(indices, delta_t);
        }
    }

    pub fn resolve_normal_forces_chunk(&mut self, indices: Vec<usize>, delta_t: f64) {
        let dimension = indices.len();
        let mut z = Mat::zeros(dimension, 2);
        let mut m: Mat<f64> = Mat::zeros(dimension, dimension);
        let mut rel_vels = Vec::with_capacity(dimension);
        for (ci, c) in indices.iter().enumerate() {
            let (i, j) = self.collision_pairs[*c].clone();
            let restitution = (i.restitution * j.restitution).sqrt();
            let (normal_c, pi, pj) = match self.collision_reports[*c] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, p1, p2, .. } => (normal, p1, p2),
            };
            z[(ci, 0)] = (i.vel + i.ori * i.ang_vel.cross(pi)).dot(normal_c);
            z[(ci, 0)] -= (j.vel + j.ori * j.ang_vel.cross(pj)).dot(normal_c);
            z[(ci, 0)] *= -(1. + restitution);

            z[(ci, 1)] = (i.forces/i.mass + i.ori * (i.moi.mul_inv(i.torques).cross(pi))).dot(normal_c);
            z[(ci, 1)] -= (j.forces/j.mass + j.ori * (j.moi.mul_inv(j.torques).cross(pj))).dot(normal_c);
            z[(ci, 1)] *= -1.;

            for (cpi, cp) in indices.iter().enumerate() {
                if cpi > ci {break;}
                let (normal_cp, pip, pjp) = match self.collision_reports[*cp] {
                    CollisionReport::None => unreachable!(),
                    CollisionReport::Some { normal, p1, p2, .. } => (normal, p1, p2),
                };
                let (ip, jp) = self.collision_pairs[*cp].clone();
                let normal_cp_i = i.ori.invert() * normal_cp;
                let normal_cp_j = j.ori.invert() * normal_cp;
                if i == ip {
                    m[(ci,cpi)] += (normal_cp / i.mass + i.ori*i.moi.mul_inv(pip.cross(normal_cp_i)).cross(pi)).dot(normal_c);
                }
                if i == jp {
                    m[(ci,cpi)] -= (normal_cp / i.mass + i.ori*i.moi.mul_inv(pip.cross(normal_cp_i)).cross(pi)).dot(normal_c);
                }
                if j == ip {
                    m[(ci,cpi)] -= (normal_cp / j.mass + j.ori*j.moi.mul_inv(pjp.cross(normal_cp_j)).cross(pj)).dot(normal_c);
                }
                if j == jp {
                    m[(ci,cpi)] += (normal_cp / j.mass + j.ori*j.moi.mul_inv(pjp.cross(normal_cp_j)).cross(pj)).dot(normal_c);
                }
                m[(cpi,ci)] = m[(ci,cpi)];
            }
            // Note M is supposed to be symmetric and positive definite
            
            let mut rel_vel = i.vel + i.ori * i.ang_vel.cross(pi);
            rel_vel -= j.vel + j.ori * j.ang_vel.cross(pj);
            rel_vel -= normal_c * rel_vel.dot(normal_c);
            rel_vels.push(rel_vel);
        }

        m.llt(Side::Upper).unwrap().solve_in_place(&mut z);

        for (ci, c) in indices.iter().enumerate() {
            let mut i = self.collision_pairs[*c].0.clone();
            let mut j = self.collision_pairs[*c].1.clone();
            let (normal, pi, pj) = match self.collision_reports[*c] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, p1, p2, .. } => (normal, p1, p2),
            };
            
            let coeff = (i.kinetic_coeff * j.kinetic_coeff).sqrt();

            let force_mag = z[(ci, 0)] / delta_t + z[(ci, 1)].max(0.);
            let mut force = force_mag * normal;
            let rel_vel_mag = rel_vels[ci].magnitude();
            if rel_vel_mag > 0. {
                // TODO implement maximum friction
                let friction_mag = coeff * force_mag;
                force += rel_vels[ci]/rel_vel_mag * friction_mag;
            }
            let inv_ori_i = i.ori.invert();
            let inv_ori_j = j.ori.invert();

            i.add_force(force);
            j.add_force(-force);
            i.add_torque(pi.cross(inv_ori_i * force));
            j.add_torque(pj.cross(inv_ori_j * -force));
        }
    }
}