use crate::{Entity, Mesh, Octonion, Quaternion};
use num_traits::{identities::One, Float, Zero};

impl<T: Float + From<f32>> Default for Entity<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Float + From<f32>> Entity<T> {
    pub fn new() -> Self {
        Self {
            pos:    Octonion::one(),
            orient: Quaternion::zero(),
            model:  Mesh::new(),
            len:    0,
        }
    }

    pub fn rotate(&mut self) -> &Self {
        self.pos.q1 = self.orient * self.pos * self.orient.conj();
        self
    }

    pub fn pos_as_arr(&self) -> [T; 8] { self.pos.as_array() }

    pub fn as_vec(&self) -> &Vec<[T; 3]> { &self.model.positions }

    pub fn add_model(
        &mut self,
        m: Mesh<T>,
    ) -> &mut Self {
        //TODO: make sure this works in all cases
        self.len = m.positions.len();
        self.model = m;
        self
    }

    pub fn set_scale(
        mut self,
        new_scale: f32,
    ) -> Self {
        self.model.scale = new_scale;
        self
    }
}
