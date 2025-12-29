use crate::graphics::CHUNK_SIZE;

use super::algo::ColliderType;


#[derive(Debug, Clone)]
pub struct RayData {
    
}

#[derive(Debug, Clone)]
pub struct BoxData {
    
}


#[derive(Debug, Clone)]
pub struct ObjectData {
    pub chunks: Vec<[u16; (CHUNK_SIZE*CHUNK_SIZE) as usize]>,
    pub coords: Vec<(i32, i32, i32)>,
}


#[derive(Clone)]
pub(super) enum ColliderIterator<'a> {
    BoxIterator {
        data: &'a BoxData
    },
    ObjectIterator {
        data: &'a ObjectData
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

    pub fn new_object(data: &'a ObjectData) -> Self {
        Self::ObjectIterator {
            data,
        }
    }

    pub fn new_ray(data: &'a RayData) -> Self {
        Self::RayIterator {
            data,
        }
    }

    pub fn next(&self) -> Vec<Self> {
        match self {
            ColliderIterator::BoxIterator { data } => todo!(),
            ColliderIterator::ObjectIterator { data } => todo!(),
            ColliderIterator::RayIterator { data } => todo!(),
        }
    }

    pub fn collider(&self) -> ColliderType {
        match self {
            ColliderIterator::BoxIterator { data } => todo!(),
            ColliderIterator::ObjectIterator { data } => todo!(),
            ColliderIterator::RayIterator { data } => todo!(),
        }
    }
}