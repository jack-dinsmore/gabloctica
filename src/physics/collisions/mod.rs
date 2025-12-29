mod algo;
pub mod shapes;

use cgmath::Vector3;
use std::cmp::Ordering;
use crate::physics::{RigidBody, collisions::shapes::ObjectData};
use algo::ColliderType;
use shapes::ColliderIterator;

/// Note: Greater reports are collided more deeply
#[derive(Debug)]
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
            (CollisionReport::Some { depth: depth1, .. }, CollisionReport::Some { depth: depth2, .. }) => {
                depth1.partial_cmp(&depth2)
            },
            (CollisionReport::None, CollisionReport::Some { .. }) => Some(Ordering::Less),
            (CollisionReport::Some { .. }, CollisionReport::None) => Some(Ordering::Greater),
        }
    }
}
impl PartialEq for CollisionReport {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CollisionReport::None, CollisionReport::None) => true,
            (CollisionReport::Some {
                depth: depth1, normal: normal1, p1: p11, p2: p21
            }, CollisionReport::Some {
                depth: depth2, normal: normal2, p1: p12, p2: p22
            }) => {
                normal1 == normal2 && depth1 == depth2 && p11 == p12 && p21 == p22
            },
            _ => false
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
    pub fn check_collision(a: &RigidBody, b: &RigidBody) -> CollisionReport {
        match (&a.collider, &b.collider) {
            (Some(ac), Some(bc)) => {
                let mut report = CollisionReport::None;
                let mut col_stack = vec![(ac.iter(), bc.iter())];
                loop {
                    match col_stack.pop() {
                        Some((ai, bi)) => {
                            let this_report = ColliderType::check_collision(&ai.collider(), &bi.collider());
                            if this_report.is_some() && this_report > report {
                                report = this_report;
                                let a_children = ai.next();
                                let b_children = bi.next();
                                for a_child in &a_children {
                                    for b_child in &b_children {
                                        col_stack.push((a_child.clone(), b_child.clone()));
                                    }
                                }
                            }
                        },
                        None => break,
                    }
                }
                report
            }
            _ => CollisionReport::None
        }
    }

    fn iter(&self) -> ColliderIterator<'_> {
        match self {
            Collider::Ray(d) => ColliderIterator::new_ray(d),
            Collider::Box(d) => ColliderIterator::new_box(d),
            Collider::Object(d) => ColliderIterator::new_object(d),
        }
    }
    
    pub fn empty_object() -> Self {
        Self::Object(ObjectData {
            chunks: Vec::new(),
            coords: Vec::new(),
        })
    }
    
    pub fn new_ray(pos: Vector3<f64>, dir: Vector3<f64>) -> Collider {
        Collider::Ray(shapes::RayData{pos, dir})
    }
}