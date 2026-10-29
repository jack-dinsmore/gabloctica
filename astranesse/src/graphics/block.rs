use cgmath::{Quaternion, Vector3};
use crate::game::object::{BlockProperties, GraphicId, chunk::PhysicalBlock};

#[derive(Clone, Copy, Debug)]
pub struct GraphicalBlock {
    pub id: GraphicId,
    ori: u8,
}
impl Default for GraphicalBlock {
    fn default() -> Self {
        Self { id: 0, ori: 0 }
    }
}
impl GraphicalBlock {
    pub fn new(id: GraphicId) -> Self {
        Self {
            id,
            ori: 0,
        }
    }
    
    pub fn from_physical(block: PhysicalBlock, properties: &BlockProperties) -> GraphicalBlock {
        if block.id == 0 {
            Self::default()
        } else {
            Self {
                id: properties[block.id].graphic_id,
                ori: block.ori,
            }
        }
    }
    
    pub(crate) fn is_opaque(&self) -> bool {
        if self.id == 0 { return false; }
        if self.id > 0x100 {return false; }
        true
    }
}

// Orientations convert to quaternions through::

impl GraphicalBlock {
    /// Get the orientation of a block given the direction the player was facing when it was placed
    pub fn ori(forward: Vector3<f32>, up: Vector3<f32>) -> u8 {
        let l_inf_norm = forward.x.abs().max(forward.y.abs().max(forward.z.abs()));
        let axis = if l_inf_norm == forward.x {0}
        else if l_inf_norm == -forward.x {1}
        else if l_inf_norm == forward.y {2}
        else if l_inf_norm == -forward.y {3}
        else if l_inf_norm == forward.z {4}
        else {5};

        let rotation = 0; // TODO use the up vector to get the orientation

        axis << 2
    }

    pub fn quat(&self) -> Quaternion<f64> {
        match self.ori {
            0 => Quaternion::new(0.500000, 0.500000, 0.500000, 0.500000),
            1 => Quaternion::new(0.000000, 0.707107, 0.707107, 0.000000),
            2 => Quaternion::new(-0.500000, 0.500000, 0.500000, -0.500000),
            3 => Quaternion::new(-0.707107, 0.000000, 0.000000, -0.707107),
            4 => Quaternion::new(-0.500000, 0.500000, -0.500000, 0.500000),
            5 => Quaternion::new(-0.707107, 0.000000, 0.000000, 0.707107),
            6 => Quaternion::new(-0.500000, -0.500000, 0.500000, 0.500000),
            7 => Quaternion::new(-0.000000, -0.707107, 0.707107, 0.000000),
            8 => Quaternion::new(0.000000, 0.707107, 0.000000, 0.707107),
            9 => Quaternion::new(-0.500000, 0.500000, 0.500000, 0.500000),
            10 => Quaternion::new(-0.707107, 0.000000, 0.707107, 0.000000),
            11 => Quaternion::new(-0.500000, -0.500000, 0.500000, -0.500000),
            12 => Quaternion::new(0.707107, 0.000000, 0.707107, 0.000000),
            13 => Quaternion::new(0.500000, 0.500000, 0.500000, -0.500000),
            14 => Quaternion::new(0.000000, 0.707107, 0.000000, -0.707107),
            15 => Quaternion::new(-0.500000, 0.500000, -0.500000, -0.500000),
            16 => Quaternion::new(0.000000, 0.000000, 0.000000, 1.000000),
            17 => Quaternion::new(0.000000, 0.000000, 0.707107, 0.707107),
            18 => Quaternion::new(0.000000, 0.000000, 1.000000, 0.000000),
            19 => Quaternion::new(0.000000, 0.000000, 0.707107, -0.707107),
            20 => Quaternion::new(0.000000, 1.000000, 0.000000, 0.000000),
            21 => Quaternion::new(-0.707107, 0.707107, 0.000000, 0.000000),
            22 => Quaternion::new(-1.000000, 0.000000, 0.000000, 0.000000),
            23 => Quaternion::new(-0.707107, -0.707107, 0.000000, -0.000000),
            _ => unreachable!()
        }
    }

    pub fn is_null(&self) -> bool {
        self.id == 0
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_forward(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22) ),
        1 => ( ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22) ),
        2 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        3 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        4 => ( ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        5 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        6 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        7 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        8 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        9 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        10 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        11 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        12 => ( ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22) ),
        13 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        14 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        15 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        16 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        17 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        18 => ( ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        19 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        20 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22) ),
        21 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        22 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        23 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_backward(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        1 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        2 => ( ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        3 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        4 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        5 => ( ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22) ),
        6 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22) ),
        7 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        8 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        9 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        10 => ( ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22) ),
        11 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        12 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        13 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        14 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        15 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        16 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22) ),
        17 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        18 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        19 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        20 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        21 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        22 => ( ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        23 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_left(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        1 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22) ),
        2 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        3 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        4 => ( ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22) ),
        5 => ( ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        6 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        7 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        8 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        9 => ( ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22) ),
        10 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        11 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        12 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        13 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        14 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        15 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        16 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        17 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        18 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        19 => ( ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        20 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        21 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22) ),
        22 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        23 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_right(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22) ),
        1 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        2 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        3 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22) ),
        4 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        5 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        6 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        7 => ( ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        8 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        9 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        10 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        11 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        12 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        13 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        14 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        15 => ( ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 0)<<18) | (0<<22) ),
        16 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        17 => ( ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        18 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        19 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        20 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        21 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        22 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        23 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 1)<<18) | (6<<22) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_up(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        1 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        2 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        3 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        4 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        5 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        6 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        7 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        8 => ( ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22) ),
        9 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22) ),
        10 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        11 => ( ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        12 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        13 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        14 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        15 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        16 => ( ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        17 => ( ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22) ),
        18 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22) ),
        19 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        20 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        21 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        22 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        23 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_down(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        1 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        2 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        3 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        4 => ( ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22) ),
        5 => ( ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        6 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        7 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 1)<<18) | (3<<22), ((self.id as u32 + 0)<<18) | (3<<22) ),
        8 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        9 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22) ),
        10 => ( ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (4<<22) ),
        11 => ( ((self.id as u32 + 1)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (4<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        12 => ( ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22) ),
        13 => ( ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22) ),
        14 => ( ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (5<<22) ),
        15 => ( ((self.id as u32 + 0)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (5<<22), ((self.id as u32 + 1)<<18) | (6<<22), ((self.id as u32 + 0)<<18) | (6<<22) ),
        16 => ( ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        17 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        18 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (2<<22) ),
        19 => ( ((self.id as u32 + 0)<<18) | (2<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (2<<22) ),
        20 => ( ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22) ),
        21 => ( ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22) ),
        22 => ( ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (1<<22) ),
        23 => ( ((self.id as u32 + 1)<<18) | (1<<22), ((self.id as u32 + 1)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (0<<22), ((self.id as u32 + 0)<<18) | (1<<22) ),
        _ => unreachable!()
}
    }
}