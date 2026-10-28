use cgmath::{Quaternion, Vector3, Zero};

use crate::{game::object::{BlockGlobalPos, ObjectIdentifier, Planet, Ship}, physics::{Physics, RigidBody, RigidBodyData, RigidBodyInit, shapes::BoxData}, util::CHUNK_SIZE};

pub struct Entity {
    pub body: RigidBody,
    pub chair: Option<(ObjectIdentifier, BlockGlobalPos, Vector3<f64>)>,
}

impl Entity {
    pub fn new(physics: &mut Physics, mut init: RigidBodyInit) -> Self {
        init.collider = Some(crate::physics::Collider::Box(BoxData { 
            center: Vector3::zero(),
            edges: Vector3::new(0.4, 0.4, 0.9), // Remember that these lengths are halfwidths
        }));
        init.moi = crate::physics::MoI::new_diagonal(Vector3::new(1e8, 1e8, 1e8));
        Self {
            body: RigidBodyData::new(&mut physics.rb_vendor, init),
            chair: None,
        }
    }

    pub fn set_chair(&mut self, id: ObjectIdentifier, block_key: BlockGlobalPos) {
        let block_pos = Vector3::new(
            block_key.0.0 as f64 * CHUNK_SIZE as f64 + block_key.1.0 as f64 + 0.5,
            block_key.0.1 as f64 * CHUNK_SIZE as f64 + block_key.1.1 as f64 + 0.5,
            block_key.0.2 as f64 * CHUNK_SIZE as f64 + block_key.1.2 as f64 + 0.5,
        );
        self.chair = Some((id, block_key, block_pos));
    }

    pub fn walk(&mut self, dir: Vector3<f64>) {
        self.body.add_force(dir);
    }

    pub fn update(&mut self, delta_t: f64, ships: &[Ship], planets: &[Planet]) {
        self.body.ori = Quaternion::new(1., 0., 0., 0.);
        if let Some((id, block_key, pos)) = &self.chair {
            let rb = &id.get_object(ships, planets).body;
            self.body.pos = rb.pos + rb.ori * (pos - rb.com_pos);
            self.body.ori = rb.ori;
        }
    }
}