mod algo;
pub mod shapes;

use cgmath::Vector3;
use rustc_hash::FxHashMap;
use std::cmp::Ordering;
use crate::physics::{RigidBody, collisions::shapes::ObjectData};
use algo::ColliderType;
use shapes::ColliderIterator;

/// Note: Greater reports are collided more deeply
#[derive(Debug,PartialEq)]
pub enum CollisionReport {
    None,
    Some {
        /// Normal vector pointing out of body 1 in inertial space
        normal: Vector3<f64>,
        /// Depth of body 1 in body 2 along the normal vector
        depth: f64,
        /// Position of the collided vertex of body 1 in body space
        p1: Vector3<f64>,
        /// Position of the collided vertex of body 2 in body space
        p2: Vector3<f64>,
    }
}
impl CollisionReport {
    pub fn is_some(&self) -> bool {
        match self {
            CollisionReport::None => false,
            CollisionReport::Some {..} => true,
        }
    }
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
}
impl PartialOrd for CollisionReport {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (CollisionReport::None, CollisionReport::None) => Some(Ordering::Equal),
            (CollisionReport::None, CollisionReport::Some { .. }) => Some(Ordering::Less),
            (CollisionReport::Some { .. }, CollisionReport::None) => Some(Ordering::Greater),
            (CollisionReport::Some { depth: d1, .. }, CollisionReport::Some { depth: d2, .. }) => d1.partial_cmp(&d2),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Collider {
    Ray(shapes::RayData),
    Box(shapes::BoxData),
    Object(shapes::ObjectData),
}

impl Collider {
    /// Check for a collision between two rigid bodies.
    pub fn check_collision(a: &RigidBody, b: &RigidBody) -> CollisionReport {
        match (&a.collider, &b.collider) {
            (Some(ac), Some(bc)) => {
                let mut report = CollisionReport::None;
                let mut col_stack = Vec::new();
                let a_children = ac.iter();
                let b_children = bc.iter();
                for a_child in &a_children {
                    for b_child in &b_children {
                        col_stack.push((a_child.clone(), b_child.clone()));
                    }
                }

                loop {
                    match col_stack.pop() {
                        Some((ai, bi)) => {
                            let this_report = ColliderType::check_collision(&ai.collider(), &bi.collider(), a, b);
                            if this_report.is_some() {
                                if ai.is_leaf() && bi.is_leaf() {
                                    // This was the last collision in the tree and it was a success
                                    if this_report > report {
                                        report = this_report;
                                    }
                                } else {
                                    // The collision was a success but it wasn't the last
                                    let a_children = ai.next();
                                    let b_children = bi.next();
                                    for a_child in &a_children {
                                        for b_child in &b_children {
                                            col_stack.push((a_child.clone(), b_child.clone()));
                                        }
                                    }
                                }
                            }
                        },
                        None => {
                            // The queue ended
                            break
                        },
                    }
                }
                report
            }
            _ => CollisionReport::None
        }
    }

    fn iter(&self) -> Vec<ColliderIterator<'_>> {
        match self {
            Collider::Ray(d) => vec![ColliderIterator::new_ray(d)],
            Collider::Box(d) => vec![ColliderIterator::new_box(d)],
            Collider::Object(d) => d.chunks.keys().map(|c| ColliderIterator::new_object(d, *c)).collect(),
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