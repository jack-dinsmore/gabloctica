use core::f64;

use cgmath::{InnerSpace, Quaternion, Rotation, Vector3, Zero};
use crate::physics::RigidBody;

use super::CollisionReport;

/// Returns None if there is no overlap, and Some(sepn) if there is overlap by distance sepn.
fn get_sepn(a: &[f64], b: &[f64]) -> Option<f64> {
    let mut min_a = a[0];
    let mut max_a = a[0];
    let mut min_b = b[0];
    let mut max_b = b[0];
    for el in a {
        min_a = min_a.min(*el);
        max_a = max_a.max(*el);
    }
    for el in b {
        min_b = min_b.min(*el);
        max_b = max_b.max(*el);
    }
    if min_a < min_b && max_a > min_b { // c P
        return Some(max_a - min_b);
    }
    if max_a <= max_b && min_a >= min_b {// c D
        return Some((max_a - min_b).abs().min((min_a - max_b).abs()))
    }
    if min_b < min_a && max_b > min_a {
        return Some(max_b - min_a);
    }
    if max_b <= max_a && min_b >= min_a {
        return Some((max_b - min_a).abs().min((min_b - max_a).abs()))
    }
    None
}

fn get_normals(ori: Quaternion<f64>) -> [Vector3<f64>; 3] {
    [
        ori * Vector3::new(1., 0., 0.),
        ori * Vector3::new(0., 1., 0.),
        ori * Vector3::new(0., 0., 1.),
    ]
}

#[derive(Debug)]
pub(super) enum ColliderType {
    Box {
        center: Vector3<f64>,
        edges: Vector3<f64>,
    },
    Ray {
        start: Vector3<f64>,
        stop: Vector3<f64>,
        dir: Vector3<f64>,
    }
}

impl ColliderType {
    pub fn check_collision(ac: &ColliderType, bc: &ColliderType, a: &RigidBody, b: &RigidBody) -> CollisionReport {
        match (ac, bc) {
            (
                ColliderType::Box { center: center1, edges: edges1 },
                ColliderType::Box { center: center2, edges: edges2 }
            ) =>  {
                unimplemented!()
            },
            (
                ColliderType::Box { center, edges },
                ColliderType::Ray { start, stop, dir }
            ) =>  {
                todo!();
            },
            (
                ColliderType::Ray { start, stop, dir },
                ColliderType::Box { center, edges }
            ) => {
                let normals = get_normals(b.ori);// TODO WRONG if the grid is rotated using global_ori
                let endpoints = [a.ori * start, a.ori * stop];

                let mut min_sepn = f64::INFINITY;
                let mut best_sepax = Vector3::zero();
                for sepax in [
                    normals[0],
                    normals[1],
                    normals[2],
                    // normals[0].cross(*dir),
                    // normals[1].cross(*dir),
                    // normals[2].cross(*dir),
                ] {
                    let dots_cube = [
                        normals[0].dot(sepax)*edges.x,
                        normals[1].dot(sepax)*edges.y,
                        normals[2].dot(sepax)*edges.z,
                    ];
                    let center_d = center.dot(sepax);
                    let dots_cube = [
                        center_d+dots_cube[0], center_d+dots_cube[1], center_d+dots_cube[2],
                        center_d-dots_cube[0], center_d-dots_cube[1], center_d-dots_cube[2],
                    ];
                    let dots_ray = endpoints.map(|v| (v + b.pos).dot(sepax));
                    match get_sepn(&dots_cube, &dots_ray) {
                        Some(sepn) => {
                            min_sepn = min_sepn.min(sepn);
                            best_sepax = sepax;
                        },
                        None => return CollisionReport::None,
                    }
                }
                
                // A collision occurred
                let cosine = best_sepax.dot(*dir);
                let collision_pos = stop - dir * min_sepn / cosine.abs();

                CollisionReport::Some {
                    normal: best_sepax,
                    depth: min_sepn,
                    p1: a.ori.invert() * (collision_pos - a.pos),
                    p2: b.ori.invert() * (collision_pos - b.pos),
                }
            },
            (ColliderType::Ray { .. }, ColliderType::Ray { .. }) => CollisionReport::None,
        }
    }
}