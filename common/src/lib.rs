#![deny(bare_trait_objects)]
pub mod entity;
pub mod octonions;
pub mod quaternions;

use num_traits::Float;

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Quaternion<T: Float> {
    val: [T; 4],
}
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Octonion<T: Float> {
    pub q1: Quaternion<T>,
    pub q2: Quaternion<T>,
}
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Entity<T: Float> {
    pos: Octonion<T>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::Octonion;
        assert_eq!(
            Octonion::new([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]) *
                Octonion::new([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
            Octonion::new([-6.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0])
        );
    }
}
