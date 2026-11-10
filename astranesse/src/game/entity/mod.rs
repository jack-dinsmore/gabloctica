use cgmath::{InnerSpace, Quaternion, Vector3, Zero};

use crate::{game::object::{BlockGlobalPos, ObjectIdentifier, Planet, Ship}, physics::{Physics, RigidBody, RigidBodyData, RigidBodyInit, shapes::BoxData}, util::CHUNK_SIZE};

pub struct Entity {
    pub body: RigidBody,
    pub chair: Option<(ObjectIdentifier, BlockGlobalPos, Vector3<f64>)>,
}

impl Entity {
    pub fn new(physics: &mut Physics, mut init: RigidBodyInit) -> Self {
        init.collider = Some(crate::physics::Collider::Box(BoxData { 
            center: Vector3::zero(),
            edges: Vector3::new(0.4, 0.4, 1.4), // Remember that these lengths are halfwidths
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

    fn get_my_planet<'a>(&self, planets: &'a [Planet]) -> Option<&'a Planet> {
        let camera_pos = self.body.pos.cast().unwrap();
        if !planets.is_empty() {
            let mut min_dist = f64::INFINITY;
            let mut min_index = 0;
            for (i, planet) in planets.iter().enumerate() {
                let o = planet.object.body.clone();
                let dist = (o.pos - camera_pos).magnitude();
                if dist < min_dist {
                    min_index = i;
                    min_dist = dist;
                }
            }
            Some(&planets[min_index])
        } else {
            None
        }
    }

    pub fn update(&mut self, delta_t: f64, physics: &Physics, ships: &[Ship], planets: &[Planet]) {
        if let Some((id, block_key, pos)) = &self.chair {
            let rb = &id.get_object(ships, planets).body;
            self.body.pos = rb.pos + rb.ori * (pos - rb.com_pos);
            self.body.ori = rb.ori;
        }

        if let Some(planet) = self.get_my_planet(planets) {
            if physics.contacting(&self.body, &planet.object.body) {
                let up = self.body.ori * Vector3::new(0., 0., 1.);
                let planet_normal = planet.object.body.ori * match planet.get_face_index(self.body.pos) {
                    0 => Vector3::new(1., 0., 0.),
                    1 => Vector3::new(-1., 0., 0.),
                    2 => Vector3::new(0., 1., 1.),
                    3 => Vector3::new(0., -1., -1.),
                    4 => Vector3::new(0., 0., 1.),
                    5 => Vector3::new(0., 0., -1.),
                    _ => unreachable!()
                };
                let mut cross = up.cross(planet_normal);
                let mag = cross.magnitude();
                if mag > 0.001 {
                    cross *= mag.min(1.5*delta_t) / mag;
                    let cos = (1. - cross.magnitude2()).sqrt();
                    self.body.ori = Quaternion::from_sv(1. + cos, cross).normalize() * self.body.ori;
                }
            }
        }
    }
}