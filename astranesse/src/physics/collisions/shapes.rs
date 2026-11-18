use cgmath::{InnerSpace, Vector3};
use rustc_hash::FxHashMap;

use crate::util::{BinaryIterator, CHUNK_SIZE};
use super::algo::ColliderType;


#[derive(Debug, Clone)]
pub struct RayData {
    pub pos: Vector3<f64>,
    pub dir: Vector3<f64>,
}

#[derive(Debug, Clone)]
pub struct BoxData {
    /// Center of the box
    pub center: Vector3<f64>,
    /// Distance from the center of the box to the edges along the different axes
    pub edges: Vector3<f64>,
}


#[derive(Debug, Clone)]
pub struct ObjectData {
    pub chunks: FxHashMap<(i32, i32, i32), [u32; (CHUNK_SIZE*CHUNK_SIZE) as usize]>,
}


#[derive(Clone, Debug)]
pub(super) enum ColliderIterator<'a> {
    BoxIterator {
        data: &'a BoxData
    },
    ObjectIterator {
        data: &'a ObjectData,
        chunk_coord: (i32, i32, i32),
        start_x: u16,
        start_y: u16,
        start_z: u16,
        width_x: u16,
        width_y: u16,
        width_z: u16,
        xyz_index: u8,
    },
    RayIterator {
        data: &'a RayData
    }
}
impl<'a> ColliderIterator<'a> {
    pub fn new_box(data: &'a BoxData) -> Self {
        Self::BoxIterator {
            data,
        }
    }

    pub fn new_object(data: &'a ObjectData, coord: (i32, i32, i32)) -> Self {
        Self::ObjectIterator {
            data,
            chunk_coord: coord,
            start_x: 0,
            start_y: 0,
            start_z: 0,
            width_x: CHUNK_SIZE as u16,
            width_y: CHUNK_SIZE as u16,
            width_z: CHUNK_SIZE as u16,
            xyz_index: 0,
        }
    }

    pub fn new_ray(data: &'a RayData) -> Self {
        Self::RayIterator {
            data,
        }
    }

    /// Returns true if the iterator contains blocks. This is a helper function for object iterators only
    pub fn contains_blocks(&self) -> bool {
        match self {
            ColliderIterator::ObjectIterator { data, chunk_coord, start_x, start_y, start_z, width_x, width_y, width_z, .. } => {
                for z in *start_z..(*start_z + *width_z) {
                    for y in *start_y..(*start_y + *width_y) {
                        let row = data.chunks[&chunk_coord][(y + z * CHUNK_SIZE as u16) as usize];
                        let mut row_mask = 0;
                        for x in *start_x..(*start_x + *width_x) {
                            row_mask |= 1 << x;
                        }
                        if row & row_mask != 0 {return true;}
                    }
                }
                false
            },
            _ => true,
        }
    }

    /// Get the collider corresponding to the iterator
    pub fn collider(&self) -> ColliderType {
        match self {
            ColliderIterator::BoxIterator { data } => ColliderType::Box {
                center: data.center,
                edges: data.edges,
            },
            ColliderIterator::ObjectIterator { data, chunk_coord, start_x, start_y, start_z, width_x, width_y, width_z, .. } => ColliderType::Box {
                center: Vector3::new(
                    chunk_coord.0 as f64*CHUNK_SIZE as f64 + (*start_x as f64 + *width_x as f64 / 2.),
                    chunk_coord.1 as f64*CHUNK_SIZE as f64 + (*start_y as f64 + *width_y as f64 / 2.),
                    chunk_coord.2 as f64*CHUNK_SIZE as f64 + (*start_z as f64 + *width_z as f64 / 2.),
                ),
                edges: Vector3::new(
                    *width_x as f64 / 2.,
                    *width_y as f64 / 2.,
                    *width_z as f64 / 2.,
                ),
            },
            ColliderIterator::RayIterator { data } => ColliderType::Ray {
                start: data.pos,
                stop: data.pos + data.dir,
                dir: data.dir.normalize()
            },
        }
    }
}

impl<'a> BinaryIterator for ColliderIterator<'a> {
    fn left(&self) -> Option<Self> {
        let mut copy = self.clone();
        match &mut copy {
            ColliderIterator::BoxIterator { data } => None,
            ColliderIterator::RayIterator { data } => None,
            ColliderIterator::ObjectIterator { data, chunk_coord, start_x, start_y, start_z, width_x, width_y, width_z, xyz_index } => {
                match *xyz_index {
                    0 => if *width_x == 1 {return None; /* leaf */ } else { *width_x /= 2; },
                    1 => if *width_y == 1 {return None; /* leaf */ } else { *width_y /= 2; },
                    2 => if *width_z == 1 {return None; /* leaf */ } else { *width_z /= 2; },
                    _ => unreachable!()
                }
                *xyz_index = (*xyz_index + 1) % 3;
                if copy.contains_blocks() {
                    Some(copy)
                } else {
                    None
                }
            },
        }
    }

    fn right(&self) -> Option<Self> {
        let mut copy = self.clone();
        match &mut copy {
            ColliderIterator::BoxIterator { data } => None,
            ColliderIterator::RayIterator { data } => None,
            ColliderIterator::ObjectIterator { data, chunk_coord, start_x, start_y, start_z, width_x, width_y, width_z, xyz_index } => {
                match *xyz_index {
                    0 => if *width_x == 1 {return None; /* leaf */ } else { *width_x /= 2; *start_x += *width_x; },
                    1 => if *width_y == 1 {return None; /* leaf */ } else { *width_y /= 2; *start_y += *width_y; },
                    2 => if *width_z == 1 {return None; /* leaf */ } else { *width_z /= 2; *start_z += *width_z; },
                    _ => unreachable!()
                }
                *xyz_index = (*xyz_index + 1) % 3;
                if copy.contains_blocks() {
                    Some(copy)
                } else {
                    None
                }
            },
        }
    }
}