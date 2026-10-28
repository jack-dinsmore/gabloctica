use core::f64;

use cgmath::{Array, InnerSpace, Quaternion, Rotation, Vector3, Zero};
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

/// Return the normals of a cube with a given orientation
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
        /// Center should be in graphical coordinates, meaning center - com_pos is the body coordinate.
        center: Vector3<f64>,
        /// Center should be in graphical coordinates, and have length equal to half the edge length of the cube
        edges: Vector3<f64>,
    },
    Ray {
        start: Vector3<f64>,
        stop: Vector3<f64>,
        dir: Vector3<f64>,
    }
}

/// Get the dot products between the eight edges and the normals
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

/// Get the coordinate of a vertex of a cube
fn get_pos_from_i(i: usize, normals: &[Vector3<f64>], edges: &Vector3<f64>) -> Vector3<f64> {
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

// Returns true if the pos is inside a cube with normals and edges as provided
fn check_inside(pos: Vector3<f64>, center: Vector3<f64>, normals: &[Vector3<f64>], edges: Vector3<f64>) -> bool {
    let d = pos - center;
    d.dot(normals[0]).abs() <= edges.x + 1e-9
    && d.dot(normals[1]).abs() <= edges.y + 1e-9
    && d.dot(normals[2]).abs() <= edges.z + 1e-9
}

impl ColliderType {
    pub fn check_collision(ac: &ColliderType, bc: &ColliderType, a: &RigidBody, b: &RigidBody) -> CollisionReport {
        match (ac, bc) {
            (
                ColliderType::Box { center: a_center_raw, edges: a_edges },
                ColliderType::Box { center: b_center_raw, edges: b_edges }
            ) =>  {
                let a_global_center = a.pos + a.ori * (a_center_raw - a.com_pos);
                let b_global_center = b.pos + b.ori * (b_center_raw - b.com_pos);
                let a_normals = get_normals(a.ori);
                let b_normals = get_normals(b.ori);
                let sepaxes = [
                    a_normals[0],
                    a_normals[1],
                    a_normals[2],
                    b_normals[0],
                    b_normals[1],
                    b_normals[2],
                    a_normals[0].cross(b_normals[0]).normalize(),
                    a_normals[0].cross(b_normals[1]).normalize(),
                    a_normals[0].cross(b_normals[2]).normalize(),
                    a_normals[1].cross(b_normals[0]).normalize(),
                    a_normals[1].cross(b_normals[1]).normalize(),
                    a_normals[1].cross(b_normals[2]).normalize(),
                    a_normals[2].cross(b_normals[0]).normalize(),
                    a_normals[2].cross(b_normals[1]).normalize(),
                    a_normals[2].cross(b_normals[2]).normalize(),
                ];

                let mut best_sepax_index = 0;
                let mut best_sepn = f64::INFINITY;
                for (i, sepax) in sepaxes.iter().enumerate() {
                    if !sepax.is_finite() { continue; }
                    let dots_a = get_dots(&a_normals, *sepax, *a_edges, a_global_center);
                    let dots_b = get_dots(&b_normals, *sepax, *b_edges, b_global_center);
                    match get_sepn(&dots_a, &dots_b) {
                        Some(sepn) => {
                            if sepn < best_sepn {
                                best_sepn = sepn;
                                best_sepax_index = i;
                            }
                        },
                        None => {
                            return CollisionReport::None
                        },
                    }
                }
                // A collision occurred with normal "best_sepax"

                let global_pos = if best_sepax_index < 6 {
                    // Vertex collision
                    let mut pos = Vector3::zero();
                    let mut n = 0;
                    for i in 0..8 {
                        let v = get_pos_from_i(i, &a_normals, &a_edges) + a_global_center;
                        if check_inside(v, b_global_center, &b_normals, *b_edges) {
                            pos += v;
                            n += 1;
                        }
                        let v = get_pos_from_i(i, &b_normals, &b_edges) + b_global_center;
                        if check_inside(v, a_global_center, &a_normals, *a_edges) {
                            pos += v;
                            n += 1;
                        }
                    }

                    if n == 0 {
                        (a_global_center + b_global_center) / 2.
                    } else {
                        pos / n as f64
                    }
                } else {
                    // Vertex collision
                    let (a_mid, a_n) = match best_sepax_index {
                        6|7|8 => (
                            a_global_center + (a_normals[1]*a_edges.y + a_normals[2]*a_edges.z) * 0.5,
                            a_normals[0],
                        ), 9|10|11 => (
                            a_global_center + (a_normals[0]*a_edges.x + a_normals[2]*a_edges.z) * 0.5,
                            a_normals[1],
                        ), 12|13|14 => (
                            a_global_center + (a_normals[0]*a_edges.x + a_normals[1]*a_edges.y) * 0.5,
                            a_normals[2],
                        ), _ => unreachable!()
                    };
                    let (b_mid, b_n) = match best_sepax_index {
                        6|9|12 => (
                            b_global_center + (b_normals[1]*b_edges.y + b_normals[2]*b_edges.z) * 0.5,
                            b_normals[0],
                        ), 7|10|13 => (
                            b_global_center + (b_normals[0]*b_edges.x + b_normals[2]*b_edges.z) * 0.5,
                            b_normals[1],
                        ), 8|11|14 => (
                            b_global_center + (b_normals[0]*b_edges.x + b_normals[1]*b_edges.y) * 0.5,
                            a_normals[2],
                        ), _ => unreachable!()
                    };
                    let r = a_mid - b_mid;
                    let a = a_n.dot(a_n);   // = 1 if a_n unit
                    let b = a_n.dot(b_n);
                    let c = b_n.dot(b_n);   // = 1 if b_n unit
                    let d = a_n.dot(r);
                    let e = b_n.dot(r);
                    let denom = a * c - b * b;

                    let s = if denom.abs() < 1e-9 {
                        // parallel edges: pick s = 0, solve t
                        0.0
                    } else {
                        (b * e - c * d) / denom
                    };

                    a_mid + a_n * s
                };
                let p1_global = global_pos - a.pos;
                let p2_global = global_pos - b.pos;

                let mut normal = sepaxes[best_sepax_index];
                if normal.dot(p1_global) < 0. {
                    normal *= -1.;
                }

                CollisionReport::Some {
                    normal,
                    depth: best_sepn,
                    p1: a.ori.invert()*p1_global,
                    p2: b.ori.invert()*p2_global,
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