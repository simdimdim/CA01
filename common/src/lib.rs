#![deny(bare_trait_objects)]
pub mod entity;
pub mod mesh;
pub mod octonions;
pub mod quaternions;

use num_traits::Float;
use vulkano::pipeline::vertex::VertexMemberTy::{self, F32, F64};

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Quaternion<T: Float> {
    pub val: [T; 4],
}
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Octonion<T: Float> {
    pub q1: Quaternion<T>,
    pub q2: Quaternion<T>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Entity<T: Float> {
    pub pos: Octonion<T>,
    pub orient: Quaternion<T>,
    pub model: Mesh<T>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh<T: Float> {
    pub rotator: Quaternion<T>,
    pub mesh: Vec<[f32; 3]>,
}

pub trait WhichFloat: Float {
    fn vmt() -> VertexMemberTy;
    fn vms() -> usize;
}
impl WhichFloat for f32 {
    fn vmt() -> VertexMemberTy {
        F32
    }

    fn vms() -> usize {
        4
    }
}
impl WhichFloat for f64 {
    fn vmt() -> VertexMemberTy {
        F64
    }

    fn vms() -> usize {
        8
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn octon() {
        use crate::Octonion;
        assert_eq!(
            Octonion::new([2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0])
                * Octonion::new([15.0, 13.0, 11.0, 9.0, 7.0, 5.0, 3.0, 1.0]),
            Octonion::new([-348.0, 92.0, 44.0, 52.0, 572.0, 28.0, 28.0, 28.0])
        );
    }
    #[test]
    fn quart() {
        use crate::Quaternion;
        use num_traits::identities::One;
        assert_eq!(
            (Quaternion::new([0.1, 0.9, 0.2, 1.0]) * Quaternion::one()).n(),
            Quaternion::new([
                -1.0,
                0.099_999_999_999_999_98,
                0.199_999_999_999_999_96,
                0.9
            ])
        );
    }
}
