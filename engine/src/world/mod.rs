use common::{Entity, Quaternion};
use num_traits::{Float, Zero};

pub struct World<T: Float> {
    origin:  Quaternion<T>,
    objects: Vec<Entity<T>>,
}

impl<T: Float> World<T> {
    pub fn new() -> Self {
        let objects = vec![];
        Self {
            origin: Quaternion::zero(),
            objects,
        }
    }

    pub fn add_object(
        &self,
        _e: Entity<T>,
    ) -> &Self {
        self
    }

    pub fn load_entities(
        &mut self,
        e: Vec<Entity<T>>,
    ) -> &Self {
        //TODO stuff
        self.objects = e;
        self
    }

    pub fn save_world(&self) {}
}
pub fn load_world<T: Float>() -> World<T> { World::new() }
