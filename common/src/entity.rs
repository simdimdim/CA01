use crate::{Entity, Mesh, Octonion, Quaternion};
use num_traits::{identities::One, Float};

impl<T: Float> Entity<T> {
    pub fn new() -> Self {
        Self {
            pos: Octonion::<T>::one(),
            orient: Quaternion::<T>::one(),
            model: Mesh::new(),
        }
    }

    pub fn rotate(&mut self) -> &Self {
        self.pos.q1 = self.orient * self.pos * self.orient.conj();
        self
    }

    pub fn pos_as_arr(&self) -> [T; 8] {
        let mut a = [T::one(); 8];
        a[0..4].copy_from_slice(&self.pos.q1.val);
        a[4..8].copy_from_slice(&self.pos.q1.val);
        a
    }
    pub fn as_vec(&self) -> &Vec<[f32; 3]> {
        &self.model.mesh
    }
}
