#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::maths::*;
        assert_eq!(
            Octonion::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0)
                * Octonion::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            Octonion::new(-6.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0)
        );
    }
}

pub mod maths {
    use nalgebra::geometry::Quaternion;
    use nalgebra::RealField;
    use num_traits::identities::One;
    use std::cmp::min;
    use std::ops::{Add, Mul, Sub};

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Octonion<T: RealField> {
        pub coords: [T; 8], // can I add ::with_capacity(8) here somehow?
    }
    impl<T: RealField> Octonion<T> {
        #[allow(clippy::too_many_arguments)]
        pub fn new(w: T, i: T, j: T, k: T, l: T, m: T, n: T, o: T) -> Self {
            Self {
                coords: [w, i, j, k, l, m, n, o],
            }
        }
        pub fn from_qs(qq1: Quaternion<T>, qq2: Quaternion<T>) -> Self {
            Self {
                coords: [
                    qq1[3], qq1[0], qq1[1], qq1[2], qq2[3], qq2[0], qq2[1], qq2[2],
                ],
            }
        }
        pub fn from_vecs(qq1: Vec<T>, qq2: Vec<T>) -> Self {
            Self {
                coords: [
                    qq1[3], qq1[0], qq1[1], qq1[2], qq2[3], qq2[0], qq2[1], qq2[2],
                ],
            }
        }
        pub fn from_vec(v: Vec<T>) -> Self {
            let mut coords = [T::zero(); 8];
            for i in 0..min(v.len(), 8) {
                coords[i] = v[i];
            }
            Self { coords }
        }
        pub fn as_vec(&self) -> Vec<T> {
            self.coords.to_vec()
        }
        pub fn lerp(self, other: Octonion<T>, t: T) -> Octonion<T> {
            &self * (Octonion::<T>::one() - t) + other * t
        }
        pub fn lerp_q(self, other: Quaternion<T>, t: T) -> Octonion<T> {
            &self * (Octonion::<T>::one() - t) + other * t
        }
        pub fn conjugate(&self) -> Self {
            Self {
                coords: [
                    self.coords[0],
                    -self.coords[1],
                    -self.coords[2],
                    -self.coords[3],
                    -self.coords[4],
                    -self.coords[5],
                    -self.coords[6],
                    -self.coords[7],
                ],
            }
        }
        pub fn conjugate_mut(&mut self) -> &Self {
            // self.coords[0] = self.coords[0];
            self.coords[1] = -self.coords[1];
            self.coords[2] = -self.coords[2];
            self.coords[3] = -self.coords[3];
            self.coords[4] = -self.coords[4];
            self.coords[5] = -self.coords[5];
            self.coords[6] = -self.coords[6];
            self.coords[7] = -self.coords[7];
            self
        }
    }
    impl<T: RealField> One for Octonion<T> {
        fn one() -> Self {
            Self::from_vec(vec![T::one(); 8])
        }
    }
    impl<T: RealField> Sub for Octonion<T> {
        type Output = Octonion<T>;

        fn sub(self, rhs: Octonion<T>) -> Octonion<T> {
            let mut coords = self.coords;
            for i in 0..8 {
                coords[i] = self.coords[i] - rhs.coords[i];
            }
            Self { coords }
        }
    }
    impl<T: RealField> Sub<T> for Octonion<T> {
        type Output = Octonion<T>;

        fn sub(self, rhs: T) -> Octonion<T> {
            let mut coords = self.coords;
            coords[0] -= rhs;
            Self { coords }
        }
    }
    impl<T: RealField> Add for Octonion<T> {
        type Output = Octonion<T>;

        fn add(self, rhs: Octonion<T>) -> Octonion<T> {
            let mut coords = self.coords;
            for i in 0..8 {
                coords[i] += rhs.coords[i];
            }
            Self { coords }
        }
    }
    impl<T: RealField> Add<Quaternion<T>> for Octonion<T> {
        type Output = Octonion<T>;

        fn add(self, rhs: Quaternion<T>) -> Octonion<T> {
            let mut coords = self.coords;
            for i in 0..4 {
                coords[i] += rhs.coords[i];
            }
            Self { coords }
        }
    }
    impl<T: RealField> Add<T> for Octonion<T> {
        type Output = Octonion<T>;

        fn add(self, rhs: T) -> Octonion<T> {
            let mut coords = self.coords;
            coords[0] += rhs;
            Self { coords }
        }
    }
    impl<T: RealField> Mul for Octonion<T> {
        type Output = Octonion<T>;

        fn mul(self, rhs: Octonion<T>) -> Octonion<T> {
            let qa = Quaternion::new(
                self.coords[0],
                self.coords[1],
                self.coords[2],
                self.coords[3],
            );
            let qb = Quaternion::new(
                self.coords[4],
                self.coords[5],
                self.coords[6],
                self.coords[7],
            );
            let qc = Quaternion::new(rhs.coords[0], rhs.coords[1], rhs.coords[2], rhs.coords[3]);
            let qd = Quaternion::new(rhs.coords[4], rhs.coords[5], rhs.coords[6], rhs.coords[7]);
            let q1 = qa * qc - qd.conjugate() * qb;
            let q2 = qd * qa + qb * qc.conjugate();
            Octonion::from_qs(q1, q2)
        }
    }
    impl<T: RealField> Mul<Octonion<T>> for &Octonion<T> {
        type Output = Octonion<T>;
        fn mul(self, rhs: Octonion<T>) -> Octonion<T> {
            let qa = Octonion::from_vec(self.coords[0..4].to_vec());
            let qb = Octonion::from_vec(self.coords[4..8].to_vec());
            let qc = Octonion::from_vec(rhs.coords[0..4].to_vec());
            let qd = Octonion::from_vec(rhs.coords[4..8].to_vec());
            let q1 = qa.clone() * qc.clone() - qd.clone().conjugate() * qb.clone();
            let q2 = qd * qa + qb * qc.conjugate();
            Octonion::from_vecs(q1.coords[0..4].to_vec(), q2.coords[4..8].to_vec())
        }
    }
    impl<T: RealField> Mul<Quaternion<T>> for Octonion<T> {
        type Output = Octonion<T>;

        fn mul(self, rhs: Quaternion<T>) -> Octonion<T> {
            self * Octonion::from(rhs)
        }
    }
    impl<T: RealField> Mul<T> for Octonion<T> {
        type Output = Octonion<T>;

        fn mul(self, rhs: T) -> Octonion<T> {
            let mut coords = self.coords;
            for i in 0..8 {
                coords[i] *= rhs;
            }
            Self { coords }
        }
    }
    impl<T: RealField> From<T> for Octonion<T> {
        fn from(o: T) -> Self {
            Self::from_vec(vec![o])
        }
    }
    impl<T: RealField> From<Quaternion<T>> for Octonion<T> {
        fn from(o: Quaternion<T>) -> Self {
            Self {
                coords: [
                    o.as_vector()[3],
                    o.as_vector()[0],
                    o.as_vector()[1],
                    o.as_vector()[2],
                    T::zero(),
                    T::zero(),
                    T::zero(),
                    T::zero(),
                ],
            }
        }
    }
}
