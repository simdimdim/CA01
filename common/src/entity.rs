use crate::{Entity, Mesh, Octonion, Quaternion};
use num_traits::{identities::One, Float, Zero};

impl<T: Float + From<f32>> Entity<T> {
    pub fn new() -> Self {
        Self {
            pos:    Octonion::one(),
            orient: Quaternion::zero(),
            model:  Mesh::new(),
        }
    }

    pub fn rotate(&mut self) -> &Self {
        self.pos.q1 = self.orient * self.pos * self.orient.conj();
        self
    }

    pub fn pos_as_arr(&self) -> [T; 8] { self.pos.as_array() }

    pub fn as_vec(&self) -> &Vec<[T; 3]> { &self.model.positions }
}
