mod algo;
pub mod shapes;

use cgmath::{InnerSpace, Vector3};
use rustc_hash::FxHashMap;
use std::cmp::Ordering;
use crate::{physics::{RigidBody, collisions::shapes::ObjectData}, util::BinaryIterator};
use algo::ColliderType;
use shapes::ColliderIterator;

/// Note: Greater reports are collided more deeply
#[derive(PartialEq, Clone)]
pub struct CollisionReport {
    pub normal: Vector3<f64>,
    pub depth: f64,
    pub p1: Vector3<f64>,
    pub p2: Vector3<f64>,
}
impl CollisionReport {
    pub fn invert(&self) -> Self {
        Self {
            normal: -self.normal,
            depth: self.depth,
            p1: self.p2,
            p2: self.p1,
        }
    }
    
    pub fn equiv(&self, other: &Self) -> bool {
        (self.normal - other.normal).magnitude2() < 1e-6 ||
        (self.p1 - other.p1).magnitude2() < 1e-6 ||
        (self.p2 - other.p2).magnitude2() < 1e-6
    }
}
impl PartialOrd for CollisionReport {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.depth.partial_cmp(&other.depth)
    }
}

impl std::fmt::Debug for CollisionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CollisionReport ({:.4}, {:.4}, {:.4}) > {:.4} | {:.4}, {:.4}, {:.4} | {:.4}, {:.4} {:.4}", self.normal.x, self.normal.y, self.normal.z, self.depth, self.p1.x, self.p1.y, self.p1.z, self.p2.x, self.p2.y, self.p2.z)
    }
}

#[derive(Debug, Clone)]
pub enum Collider {
    Ray(shapes::RayData),
    Box(shapes::BoxData),
    Object(shapes::ObjectData),
}

impl Collider {
    pub fn check_closest_collision(a: &RigidBody, b: &RigidBody) -> Option<CollisionReport> {
        let mut out = None;
        Self::check_collisions(a, b, &mut |report| {
            match &out {
                None => out = Some(report),
                Some(o) => if report > *o { out = Some(report) },
            }
        });
        out
    }

    /// Check for a collision between two rigid bodies and save all collisions to a list
    pub fn check_collisions(a: &RigidBody, b: &RigidBody, action: &mut impl FnMut(CollisionReport)) {
        if let (Some(ac), Some(bc)) = (&a.collider, &b.collider) {
            for a_iter in ac.iter() {
                for b_iter in bc.iter() {
                    Self::check_intermediate_collision(a_iter.clone(), b_iter, a, b, action, false);
                }
            }
        }
    }

    fn check_intermediate_collision(ai: ColliderIterator, bi: ColliderIterator, a: &RigidBody, b: &RigidBody, action: &mut impl FnMut(CollisionReport), inverse: bool) {
        if let Some(report) = ColliderType::check_collision(&ai.collider(), &bi.collider(), a, b) {
            // A collision occurred
            match (ai.left(), ai.right()) {
                (Some(left), Some(right)) => {
                    Self::check_intermediate_collision(bi.clone(), left, b, a, action, !inverse);
                    Self::check_intermediate_collision(bi, right, b, a, action, !inverse);
                },
                (Some(left), None) => Self::check_intermediate_collision(bi, left, b, a, action, !inverse),
                (None, Some(right)) => Self::check_intermediate_collision(bi, right, b, a, action, !inverse),
                (None, None) => {
                    // a is a leaf
                    match (bi.left(), bi.right()) {
                        (Some(left), Some(right)) => {
                            Self::check_intermediate_collision(ai.clone(), left, a, b, action, inverse);
                            Self::check_intermediate_collision(ai, right, a, b, action, inverse);
                        },
                        (Some(left), None) => Self::check_intermediate_collision(ai, left, a, b, action, inverse),
                        (None, Some(right)) => Self::check_intermediate_collision(ai, right, a, b, action, inverse),
                        (None, None) => {
                            // b is also a leaf
                            let report = if inverse {report.invert()} else {report};
                            action(report);
                        }
                    }
                }
            }
        }
    }

    fn iter(&self) -> Vec<ColliderIterator<'_>> {
        match self {
            Collider::Ray(d) => vec![ColliderIterator::new_ray(d)],
            Collider::Box(d) => vec![ColliderIterator::new_box(d)],
            Collider::Object(d) => {
                let mut output = Vec::new();
                for chunk in d.chunks.keys() {
                    let iter = ColliderIterator::new_object(d, *chunk);
                    if iter.contains_blocks() {
                        output.push(iter);
                    }
                }
                output
            }
        }
    }
    
    pub fn empty_object() -> Self {
        Self::Object(ObjectData {
            chunks: FxHashMap::default(),
        })
    }
    
    pub fn new_ray(pos: Vector3<f64>, dir: Vector3<f64>) -> Collider {
        Collider::Ray(shapes::RayData{pos, dir})
    }
}