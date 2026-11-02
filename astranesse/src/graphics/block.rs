use cgmath::Vector3;
use crate::{game::object::{BlockProperties, GraphicId, chunk::PhysicalBlock}, util::Ori};

#[derive(Clone, Copy, Debug)]
pub struct GraphicalBlock {
    pub id: GraphicId,
    ori: Ori,
}
impl Default for GraphicalBlock {
    fn default() -> Self {
        Self { id: 0, ori: Ori { data: 0 } }
    }
}
impl GraphicalBlock {
    pub fn new(id: GraphicId) -> Self {
        Self {
            id,
            ori: Ori { data: 0 },
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

    pub fn is_null(&self) -> bool {
        self.id == 0
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_forward(&self) -> (u32, u32, u32, u32) {
        match self.ori.data {
        0 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26) ),
        1 => ( ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26) ),
        2 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        3 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        4 => ( ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        5 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        6 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        7 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        8 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        9 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        10 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        11 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        12 => ( ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26) ),
        13 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        14 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        15 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        16 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        17 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        18 => ( ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        19 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        20 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26) ),
        21 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        22 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        23 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_backward(&self) -> (u32, u32, u32, u32) {
        match self.ori.data {
        0 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        1 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        2 => ( ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        3 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        4 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        5 => ( ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26) ),
        6 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26) ),
        7 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        8 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        9 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        10 => ( ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26) ),
        11 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        12 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        13 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        14 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        15 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        16 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26) ),
        17 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        18 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        19 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        20 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        21 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        22 => ( ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        23 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_left(&self) -> (u32, u32, u32, u32) {
        match self.ori.data {
        0 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        1 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26) ),
        2 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        3 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        4 => ( ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26) ),
        5 => ( ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        6 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        7 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        8 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        9 => ( ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26) ),
        10 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        11 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        12 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        13 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        14 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        15 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        16 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        17 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        18 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        19 => ( ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        20 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        21 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26) ),
        22 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        23 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_right(&self) -> (u32, u32, u32, u32) {
        match self.ori.data {
        0 => ( ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26) ),
        1 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        2 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        3 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26) ),
        4 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        5 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        6 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        7 => ( ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        8 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        9 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        10 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        11 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        12 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        13 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        14 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        15 => ( ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 0)<<18) | (0<<26) ),
        16 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        17 => ( ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        18 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        19 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        20 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        21 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        22 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        23 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 1)<<18) | (6<<26) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_up(&self) -> (u32, u32, u32, u32) {
        match self.ori.data {
        0 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        1 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        2 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        3 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        4 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        5 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        6 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        7 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        8 => ( ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26) ),
        9 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26) ),
        10 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        11 => ( ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        12 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        13 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        14 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        15 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        16 => ( ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        17 => ( ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26) ),
        18 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26) ),
        19 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        20 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        21 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        22 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        23 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_down(&self) -> (u32, u32, u32, u32) {
        match self.ori.data {
        0 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        1 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        2 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        3 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        4 => ( ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26) ),
        5 => ( ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        6 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        7 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 1)<<18) | (3<<26), ((self.id as u32 + 0)<<18) | (3<<26) ),
        8 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        9 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26) ),
        10 => ( ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (4<<26) ),
        11 => ( ((self.id as u32 + 1)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (4<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        12 => ( ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26) ),
        13 => ( ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26) ),
        14 => ( ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (5<<26) ),
        15 => ( ((self.id as u32 + 0)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (5<<26), ((self.id as u32 + 1)<<18) | (6<<26), ((self.id as u32 + 0)<<18) | (6<<26) ),
        16 => ( ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        17 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        18 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (2<<26) ),
        19 => ( ((self.id as u32 + 0)<<18) | (2<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (2<<26) ),
        20 => ( ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26) ),
        21 => ( ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26) ),
        22 => ( ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (1<<26) ),
        23 => ( ((self.id as u32 + 1)<<18) | (1<<26), ((self.id as u32 + 1)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (0<<26), ((self.id as u32 + 0)<<18) | (1<<26) ),
        _ => unreachable!()
}
    }
}