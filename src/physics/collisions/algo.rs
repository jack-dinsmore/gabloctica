use super::CollisionReport;

pub(super) enum ColliderType {
    Box {
        
    }
}

impl ColliderType {
    pub fn check_collision(a: &ColliderType, b: &ColliderType) -> CollisionReport {
        unimplemented!()
    }
}