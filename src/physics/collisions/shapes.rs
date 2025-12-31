use cgmath::{InnerSpace, Vector3};

use crate::graphics::CHUNK_SIZE;

use super::algo::ColliderType;


#[derive(Debug, Clone)]
pub struct RayData {
    pub pos: Vector3<f64>,
    pub dir: Vector3<f64>,
}

#[derive(Debug, Clone)]
pub struct BoxData {
    
}


#[derive(Debug, Clone)]
pub struct ObjectData {
    pub chunks: Vec<[u16; (CHUNK_SIZE*CHUNK_SIZE) as usize]>,
    pub coords: Vec<(i32, i32, i32)>,
}


#[derive(Clone, Debug)]
pub(super) enum ColliderIterator<'a> {
    BoxIterator {
        data: &'a BoxData
    },
    ObjectIterator {
        data: &'a ObjectData,
        chunk_index: usize,
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

    pub fn new_object(data: &'a ObjectData, index: usize) -> Self {
        Self::ObjectIterator {
            data,
            chunk_index: index,
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

    /// This function is called only if a collision occurred
    pub fn next(&self) -> Vec<Self> {
        let mut copy = self.clone();
        match &copy {
            ColliderIterator::ObjectIterator {xyz_index, ..} => {
                // TODO check if the collision report is inside a block. If it is, return an empty vector
                // Descend deeper into the chunk
                let mut output = Vec::new();
                let old_xyz_index = *xyz_index;

                if let ColliderIterator::ObjectIterator {xyz_index, width_x, width_y, width_z, ..} = &mut copy {
                    *xyz_index = (old_xyz_index + 1) % 3;
                    match old_xyz_index {
                        0 => if *width_x == 1 { return Vec::new(); } else { *width_x /= 2; },
                        1 => if *width_y == 1 { return Vec::new(); } else { *width_y /= 2; },
                        2 => if *width_z == 1 { return Vec::new(); } else { *width_z /= 2; },
                        _ => unreachable!()
                    };
                }
                if copy.contains_blocks() { output.push(copy.clone()); }

                if let ColliderIterator::ObjectIterator {start_x, start_y, start_z, width_x, width_y, width_z, ..} = &mut copy {
                    match old_xyz_index {
                        0 => *start_x += *width_x,
                        1 => *start_y += *width_y,
                        2 => *start_z += *width_z,
                        _ => unreachable!()
                    };
                }
                if copy.contains_blocks() { output.push(copy); }
                output
            },
            _ => vec![copy],
        }
    }

    pub fn collider(&self) -> ColliderType {
        match self {
            ColliderIterator::BoxIterator { data } => unimplemented!(),
            ColliderIterator::ObjectIterator { data, chunk_index, start_x, start_y, start_z, width_x, width_y, width_z, .. } => ColliderType::Box {
                center: Vector3::new(
                    data.coords[*chunk_index].0 as f64*CHUNK_SIZE as f64 + (*start_x as f64 + *width_x as f64 / 2.),
                    data.coords[*chunk_index].1 as f64*CHUNK_SIZE as f64 + (*start_y as f64 + *width_y as f64 / 2.),
                    data.coords[*chunk_index].2 as f64*CHUNK_SIZE as f64 + (*start_z as f64 + *width_z as f64 / 2.),
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
    
    /// Return true if this iterator splits into smaller ones, and false if it never splits.
    pub fn is_leaf(&self) -> bool {
        match self {
            ColliderIterator::ObjectIterator { width_x, width_y, width_z, .. } => *width_x == 1 && *width_y == 1 && *width_z == 1,
            ColliderIterator::BoxIterator { .. } => true,
            ColliderIterator::RayIterator { .. } => true,
        }
    }

    /// Returns true if the iterator contains blocks. This is a helper function for object iterators only
    fn contains_blocks(&self) -> bool {
        match self {
            ColliderIterator::ObjectIterator { data, chunk_index, start_x, start_y, start_z, width_x, width_y, width_z, .. } => {
                for z in *start_z..(*start_z + *width_z) {
                    for y in *start_y..(*start_y + *width_y) {
                        let row = data.chunks[*chunk_index][(y + z * CHUNK_SIZE as u16) as usize];
                        for x in *start_x..(*start_x + *width_x) {
                            if row & (1 << x) != 0 {return true;}
                        }
                    }
                }
                false
            },
            _ => unreachable!(),
        }
    }
}