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
        for (i, mut a) in self.rb_vendor.iter().enumerate() {
            for (j, mut  b) in self.rb_vendor.iter().enumerate() {
                if j >= i {break;}
                let report = Collider::check_collision(&a, &b);
                if let CollisionReport::Some { normal, depth, p1, p2 } = report {
                    let normal1 = a.ori.invert() * normal;
                    let normal2 = b.ori.invert() * normal;
                    let denom = 
                        1. / a.mass +
                        1. / b.mass + 
                        a.moi.mul_inv(p1.cross(normal1)).cross(p1).dot(normal1) + 
                        b.moi.mul_inv(p2.cross(normal2)).cross(p2).dot(normal2)
                    ;
                    let rel_vel = (a.vel + a.ori*a.ang_vel.cross(p1)) - (b.vel + b.ori*b.ang_vel.cross(p2));
                    let restitution = (a.restitution * b.restitution).sqrt();
                    let kinetic_coeff = (a.kinetic_coeff * b.kinetic_coeff).sqrt();
                    let force = (1. + restitution) * rel_vel.dot(normal) / denom / delta_t;
                    a.add_force(-force * normal);
                    b.add_force(force * normal);
                    a.add_torque(-p1.cross(normal1) * force);
                    b.add_torque(p2.cross(normal2) * force);

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
            let torque = body.moi.get_self_torque(body.ang_vel);
            body.add_torque(torque);
        }
        self.resolve_normal_forces();

        // Force updates
        for mut body in self.rb_vendor.iter() {
            body.update(delta_t);
        }
    }

    pub fn resolve_normal_forces(&mut self) {
        if self.collision_reports.is_empty() {return;}

        let mut collision_bodies: Vec<FxHashSet<RigidBody>> = Vec::new();
        let mut collision_indices: Vec<Vec<usize>> = Vec::new();
        for (k, (i, j)) in self.collision_pairs.iter().enumerate() {
            let (normal_k, pi, pj) = match self.collision_reports[k] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, p1, p2, .. } => (normal, p1, p2),
            };
            let mut rel_accel = Vector3::new(0., 0., 0.);
            rel_accel -= (i.forces + i.ori * (i.moi.mul_inv(i.torques).cross(pi))) / i.mass;
            rel_accel += (j.forces + j.ori * (j.moi.mul_inv(j.torques).cross(pj))) / j.mass;
            if rel_accel.dot(normal_k) > 0. {
                // Do not consider pairs which are forced away from each other (normal force would be negative)
                continue;
            }

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
            self.resolve_normal_forces_chunk(indices);
        }
    }

    pub fn resolve_normal_forces_chunk(&mut self, indices: Vec<usize>) {
        let dimension = indices.len();
        let mut delta: Mat<f64> = Mat::zeros(dimension, dimension);
        let mut a = Mat::zeros(dimension, 1);
        let mut friction_dirs = Vec::new();
        for (ki, k) in indices.iter().enumerate() {
            let i = self.collision_pairs[*k].0.clone();
            let j = self.collision_pairs[*k].1.clone();
            let (normal_k, pi, pj) = match self.collision_reports[*k] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, p1, p2, .. } => (normal, p1, p2),
            };
            let mut rel_accel = Vector3::new(0., 0., 0.);
            rel_accel -= (i.forces + i.ori * (i.moi.mul_inv(i.torques).cross(pi))) / i.mass;
            rel_accel += (j.forces + j.ori * (j.moi.mul_inv(j.torques).cross(pj))) / j.mass;
            a[(ki, 0)] = rel_accel.dot(normal_k);
            let friction_dir = rel_accel - normal_k * rel_accel.dot(normal_k);
            friction_dirs.push(friction_dir.normalize());
            for (kpi, kp) in indices.iter().enumerate() {
                let normal_kp = match self.collision_reports[*k] {
                    CollisionReport::None => unreachable!(),
                    CollisionReport::Some { normal, .. } => normal,
                };

                let ip = self.collision_pairs[*kp].0.clone();
                let jp = self.collision_pairs[*kp].1.clone();
                let s = if i == ip {1.} else if i == jp {-1.} else {0.};
                delta[(ki, kpi)] += s * (normal_kp + i.ori * (i.moi.mul_inv((i.ori.invert()*normal_kp).cross(pi)))).dot(normal_k);
                let s = if j == ip {1.} else if j == jp {-1.} else {0.};
                delta[(ki, kpi)] -= s * (normal_kp + j.ori * (j.moi.mul_inv((j.ori.invert()*normal_kp).cross(pj)))).dot(normal_k);
            }
        }
        delta.llt(Side::Upper).unwrap().solve_in_place(&mut a);

        for (ki, k) in indices.iter().enumerate() {
            let mut i = self.collision_pairs[*k].0.clone();
            let mut j = self.collision_pairs[*k].1.clone();
            let (normal, pi, pj) = match self.collision_reports[*k] {
                CollisionReport::None => unreachable!(),
                CollisionReport::Some { normal, p1, p2, .. } => (normal, p1, p2),
            };
            
            let rel_vel = (i.vel + i.ori * i.ang_vel.cross(pi)) - (j.vel + j.ori * j.ang_vel.cross(pj));
            let coeff = if rel_vel.magnitude() < 0.001 {
                (i.static_coeff * j.static_coeff).sqrt()
            } else {
                (i.kinetic_coeff * j.kinetic_coeff).sqrt()
            };

            let force = a[(ki, 0)] * (normal + friction_dirs[ki] * coeff);
            let inv_ori_i = i.ori.invert();
            let inv_ori_j = j.ori.invert();

            i.add_force(force);
            j.add_force(-force);
            i.add_force(pi.cross(inv_ori_i * force));
            j.add_force(pj.cross(inv_ori_j * -force));
        }
    }
}