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

fn get_dots(normals: &[Vector3<f64>], sepax: Vector3<f64>, edges: Vector3<f64>, global_center: Vector3<f64>) -> [f64; 8] {
    let dots = [
        normals[0].dot(sepax)*edges.x,
        normals[1].dot(sepax)*edges.y,
        normals[2].dot(sepax)*edges.z,
    ];
    let center_d = global_center.dot(sepax);
    [
        center_d + dots[0] + dots[1] + dots[2],
        center_d + dots[0] + dots[1] - dots[2],
        center_d + dots[0] - dots[1] + dots[2],
        center_d + dots[0] - dots[1] - dots[2],
        center_d - dots[0] + dots[1] + dots[2],
        center_d - dots[0] + dots[1] - dots[2],
        center_d - dots[0] - dots[1] + dots[2],
        center_d - dots[0] - dots[1] - dots[2],
    ]
}

fn get_normal_from_i(i: usize, normals: &[Vector3<f64>], edges: &Vector3<f64>) -> Vector3<f64> {
    match i {
        0 =>  normals[0]*edges.x + normals[1]*edges.y + normals[2]*edges.z,
        1 =>  normals[0]*edges.x + normals[1]*edges.y - normals[2]*edges.z,
        2 =>  normals[0]*edges.x - normals[1]*edges.y + normals[2]*edges.z,
        3 =>  normals[0]*edges.x - normals[1]*edges.y - normals[2]*edges.z,
        4 => -normals[0]*edges.x + normals[1]*edges.y + normals[2]*edges.z,
        5 => -normals[0]*edges.x + normals[1]*edges.y - normals[2]*edges.z,
        6 => -normals[0]*edges.x - normals[1]*edges.y + normals[2]*edges.z,
        7 => -normals[0]*edges.x - normals[1]*edges.y - normals[2]*edges.z,
        _ => unreachable!()
    }
}

impl ColliderType {
    pub fn check_collision(ac: &ColliderType, bc: &ColliderType, a: &RigidBody, b: &RigidBody) -> CollisionReport {
        match (ac, bc) {
            (
                ColliderType::Box { center: centera, edges: edgesa },
                ColliderType::Box { center: centerb, edges: edgesb }
            ) =>  {
                let a_global_center = a.pos + a.ori * (centera - a.com_pos);
                let b_global_center = b.pos + b.ori * (centerb - b.com_pos);
                let a_normals = get_normals(a.ori);
                let b_normals = get_normals(b.ori);

                let mut best_sepax = Vector3::zero();
                let mut best_sepn = f64::INFINITY;
                for sepax in [
                    a_normals[0],
                    a_normals[1],
                    a_normals[2],
                    b_normals[0],
                    b_normals[1],
                    b_normals[2],
                    a_normals[0].cross(b_normals[0]),
                    a_normals[0].cross(b_normals[1]),
                    a_normals[0].cross(b_normals[2]),
                    a_normals[1].cross(b_normals[0]),
                    a_normals[1].cross(b_normals[1]),
                    a_normals[1].cross(b_normals[2]),
                    a_normals[2].cross(b_normals[0]),
                    a_normals[2].cross(b_normals[1]),
                    a_normals[2].cross(b_normals[2]),
                ] {
                    if sepax.magnitude2() < 1e-8 {continue;}
                    let dots_a = get_dots(&a_normals, sepax, *edgesa, a_global_center);
                    let dots_b = get_dots(&b_normals, sepax, *edgesb, b_global_center);
                    match get_sepn(&dots_a, &dots_b) {
                        Some(sepn) => {
                            if sepn < best_sepn {
                                best_sepn = sepn;
                                best_sepax = sepax;
                            }
                        },
                        None => return CollisionReport::None,
                    }
                }
                // A collision occurred

                // Get which corners collided
                let dots_a = get_dots(&a_normals, best_sepax, *edgesa, a_global_center);
                let dots_b = get_dots(&b_normals, best_sepax, *edgesb, b_global_center);
                let mut p1 = Vector3::new(0., 0., 0.);
                let mut p2 = Vector3::new(0., 0., 0.);
                let mut n_p1 = 0;
                let mut n_p2 = 0;
                let min_a = dots_a.iter().fold(dots_a[0], |a, c| a.min(*c));
                let min_b = dots_b.iter().fold(dots_b[0], |a, c| a.min(*c));
                let max_a = dots_a.iter().fold(dots_a[0], |a, c| a.max(*c));
                let max_b = dots_b.iter().fold(dots_b[0], |a, c| a.max(*c));
                for (i, el) in dots_a.iter().enumerate() {
                    if (*el < max_b) && (*el > min_b) {
                        p1 += get_normal_from_i(i, &a_normals, &edgesa);
                        n_p1 += 1;
                    }
                }
                for (i, el) in dots_b.iter().enumerate() {
                    if (*el < max_a) && (*el > min_a) {
                        p2 += get_normal_from_i(i, &b_normals, &edgesb);
                        n_p2 += 1;
                    }
                }
                p1 /= n_p1 as f64;
                p2 /= n_p2 as f64;

                if p1.dot(best_sepax) < 0. {
                    best_sepax *= -1.;
                }

                CollisionReport::Some {
                    normal: best_sepax,
                    depth: best_sepn,
                    p1: a.ori.invert() * p1 + a.com_pos,
                    p2: b.ori.invert() * p1 + b.com_pos,
                }
            },
            (
                ColliderType::Box { center, edges },
                ColliderType::Ray { start, stop, dir }
            ) =>  {
                let global_center = a.pos + a.ori * (center - a.com_pos);
                let normals = get_normals(a.ori);
                let endpoints = [start, stop];

                let mut best_alpha = f64::INFINITY;
                let mut best_sepax = Vector3::zero();
                for sepax in [
                    normals[0],
                    normals[1],
                    normals[2],
                ] {
                    let dots_cube = get_dots(&normals, sepax, *edges, global_center);
                    let dots_ray = endpoints.map(|v| v.dot(sepax));
                    match get_sepn(&dots_cube, &dots_ray) {
                        Some(sepn) => {
                            // Since this is a ray-box collision, use the normal with the closest collision point. That is, minimize alpha = sepn / |sepax.dir|.
                            let alpha = sepn / sepax.dot(*dir).abs();
                            if alpha < best_alpha {
                                best_alpha = alpha;
                                best_sepax = sepax;
                            }
                        },
                        None => return CollisionReport::None,
                    }
                }
                
                // A collision occurred
                let collision_pos = stop - dir * best_alpha;
                CollisionReport::Some {
                    normal: best_sepax,
                    depth: best_alpha,
                    p1: a.ori.invert() * (collision_pos - a.pos) + a.com_pos,
                    p2: b.ori.invert() * (collision_pos - b.pos) + b.com_pos,
                }
            },
            (
                ColliderType::Ray { start, stop, dir },
                ColliderType::Box { center, edges }
            ) => {
                Self::check_collision(bc, ac, b, a).invert()
            },
            (ColliderType::Ray { .. }, ColliderType::Ray { .. }) => CollisionReport::None,
        }
    }
}