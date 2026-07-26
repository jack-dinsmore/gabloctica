use cgmath::{Quaternion, Vector3, Zero};

use crate::{game::object::{BlockKey, Object}, graphics::CHUNK_SIZE, physics::{Physics, RigidBody, RigidBodyInit, shapes::BoxData}, util::RcCell};

pub struct Entity {
    pub body: RigidBody,
    pub chair: Option<(RcCell<Object>, BlockKey, Vector3<f64>)>,
}

impl Entity {
    pub fn new(physics: &mut Physics, mut init: RigidBodyInit) -> Self {
        init.collider = Some(crate::physics::Collider::Box(BoxData { 
            center: Vector3::zero(),
            edges: Vector3::new(0.9, 0.4, 0.4),
        }));
        init.moi = crate::physics::MoI::new_diagonal(Vector3::new(1e8, 1e8, 1e8));
        Self {
            body: RigidBody::new(&mut physics.rb_vendor, init),
            chair: None,
        }
    }

    pub fn set_chair(&mut self, object: &RcCell<Object>, block_key: BlockKey) {
        let block_pos = Vector3::new(
            block_key.0.0 as f64 * CHUNK_SIZE as f64 + block_key.1.0 as f64 + 0.5,
            block_key.0.1 as f64 * CHUNK_SIZE as f64 + block_key.1.1 as f64 + 0.5,
            block_key.0.2 as f64 * CHUNK_SIZE as f64 + block_key.1.2 as f64 + 0.5,
        );
        self.chair = Some((object.clone(), block_key, block_pos));
    }

    pub fn walk(&mut self, dir: Vector3<f64>) {
        self.body.add_force(dir);
    }

    pub fn update(&mut self, delta_t: f64) {
        self.body.ori = Quaternion::new(1., 0., 0., 0.);
        if let Some((o, block_key, pos)) = &self.chair {
            let obj = o.borrow();
            self.body.pos = obj.body.pos + obj.body.ori * (pos - obj.body.com_pos);
            self.body.ori = obj.body.ori;
        }
    }
}