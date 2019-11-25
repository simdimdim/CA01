use crate::{Mesh, Quaternion};
use num_traits::{identities::Zero, Float};

impl<T: Float> Mesh<T> {
    pub fn new() -> Self {
        Self {
            rotator: Quaternion::zero(),
            mesh:    vec![[0.3f32, 0.3f32, 0.3f32], [0.3f32, 0.3f32, 0.3f32], [
                0.3f32, 0.3f32, 0.3f32,
            ]],
        }
    }

    pub fn add_points(
        &mut self,
        inp: Vec<[f32; 3]>,
    ) -> &Self {
        self.mesh.extend(&inp);
        self
    }

    pub fn rotate(
        &mut self,
        rotator: Quaternion<T>,
    ) {
        self.rotator = rotator * self.rotator * rotator.conj()
    }
}
