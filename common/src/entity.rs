use crate::{Entity, Mesh, Octonion, Quaternion};
use num_traits::{identities::One, Float};

impl<T: Float> Entity<T> {
    pub fn new() -> Self {
        Self {
            pos:    Octonion::<T>::one(),
            orient: Quaternion::<T>::one(),
            model:  Mesh::new(),
        }
    }

    pub fn rotate(&mut self) -> &Self {
        self.pos.q1 = self.orient * self.pos * self.orient.conj();
        self
    }

    pub fn pos_as_arr(&self) -> [T; 8] { self.pos.as_array() }

    pub fn as_vec(&self) -> &Vec<[f32; 3]> { &self.model.mesh }
}
