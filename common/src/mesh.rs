use crate::{Mesh, Quaternion};
use num_traits::Float;
use tobj;
impl<T: Float + From<f32>> Mesh<T> {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            normals:   vec![],
            indices:   vec![],
            scale:     0.0,
            offset:    [T::zero(); 3],
        }
    }

    pub fn from_tobj_to_mesh(
        mesh: &tobj::Mesh,
        translation: [T; 3],
        scale: f32,
    ) -> Mesh<T> {
        let positions: Vec<[T; 3]> = mesh
            .positions
            .chunks(3)
            .map(|i| [i[0].into(), i[1].into(), i[2].into()])
            .collect();
        let normals: Vec<[T; 3]> = mesh
            .normals
            .chunks(3)
            .map(|i| [i[0].into(), i[1].into(), i[2].into()])
            .collect();
        let indices = mesh.indices.to_vec();

        Mesh {
            positions,
            normals,
            indices,
            scale,
            offset: translation,
        }
    }

    pub fn add_points(
        &mut self,
        inp: Vec<[T; 3]>,
    ) -> &Self {
        self.positions.extend(&inp);
        self
    }

    pub fn rotate(
        &mut self,
        rotator: Quaternion<T>,
    ) {
        for i in 0..self.positions.len() {
            self.positions[i] = (rotator *
                Quaternion::fom_imag(self.positions[i]) *
                rotator.conj())
            .imag();
        }
    }
}
