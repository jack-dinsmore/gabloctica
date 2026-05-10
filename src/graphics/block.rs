use cgmath::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub id: u8,
    pub ori: u8,
}

// Orientations convert to quaternions through::

impl Block {
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
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20) ),
        1 => ( ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20) ),
        2 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        3 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        4 => ( ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        5 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        6 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        7 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        8 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        9 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        10 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        11 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        12 => ( ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20) ),
        13 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        14 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        15 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        16 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        17 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        18 => ( ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        19 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        20 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20) ),
        21 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        22 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        23 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_backward(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        1 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        2 => ( ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        3 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        4 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        5 => ( ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20) ),
        6 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20) ),
        7 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        8 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        9 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        10 => ( ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20) ),
        11 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        12 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        13 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        14 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        15 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        16 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20) ),
        17 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        18 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        19 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        20 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        21 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        22 => ( ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        23 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_left(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        1 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20) ),
        2 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        3 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        4 => ( ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20) ),
        5 => ( ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        6 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        7 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        8 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        9 => ( ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20) ),
        10 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        11 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        12 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        13 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        14 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        15 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        16 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        17 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        18 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        19 => ( ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        20 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        21 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20) ),
        22 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        23 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_right(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20) ),
        1 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        2 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        3 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20) ),
        4 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        5 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        6 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        7 => ( ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        8 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        9 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        10 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        11 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        12 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        13 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        14 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        15 => ( ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 0)<<16) | (0<<20) ),
        16 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        17 => ( ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        18 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        19 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        20 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        21 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        22 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        23 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 1)<<16) | (6<<20) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_up(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        1 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        2 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        3 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        4 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        5 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        6 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        7 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        8 => ( ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20) ),
        9 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20) ),
        10 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        11 => ( ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        12 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        13 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        14 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        15 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        16 => ( ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        17 => ( ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20) ),
        18 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20) ),
        19 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        20 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        21 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        22 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        23 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        _ => unreachable!()
}
    }

    /// Get the data points representing the four corners of the the upward facing face
    pub fn get_uv_down(&self) -> (u32, u32, u32, u32) {
        match self.ori {
        0 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        1 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        2 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        3 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        4 => ( ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20) ),
        5 => ( ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        6 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        7 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 1)<<16) | (3<<20), ((self.id as u32 + 0)<<16) | (3<<20) ),
        8 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        9 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20) ),
        10 => ( ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (4<<20) ),
        11 => ( ((self.id as u32 + 1)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (4<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        12 => ( ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20) ),
        13 => ( ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20) ),
        14 => ( ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (5<<20) ),
        15 => ( ((self.id as u32 + 0)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (5<<20), ((self.id as u32 + 1)<<16) | (6<<20), ((self.id as u32 + 0)<<16) | (6<<20) ),
        16 => ( ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        17 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        18 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (2<<20) ),
        19 => ( ((self.id as u32 + 0)<<16) | (2<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (2<<20) ),
        20 => ( ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20) ),
        21 => ( ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20) ),
        22 => ( ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (1<<20) ),
        23 => ( ((self.id as u32 + 1)<<16) | (1<<20), ((self.id as u32 + 1)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (0<<20), ((self.id as u32 + 0)<<16) | (1<<20) ),
        _ => unreachable!()
}
    }
}