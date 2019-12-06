use crate::{Octonion, Quaternion, WhichFloat};
use num_traits::{Float, One, Zero};
use std::ops::{Add, Div, Mul, Neg, Sub};
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

impl<T: Float + From<f32>> Quaternion<T> {
    pub fn new(i: [T; 4]) -> Self { Self { val: i } }

    pub fn fom_imag(i: [T; 3]) -> Self {
        Self {
            val: [0.0.into(), i[0], i[1], i[2]],
        }
    }

    pub fn imag(&self) -> [T; 3] { [self.val[1], self.val[2], self.val[3]] }

    pub fn from_slice(inp: &[T]) -> Self {
        let mut q = Quaternion::zero();
        q.val.copy_from_slice(&inp);
        q
    }

    pub fn conj(self) -> Quaternion<T> {
        Self {
            val: [self.val[0], -self.val[1], -self.val[2], -self.val[3]],
        }
    }

    pub fn inv(self) -> Quaternion<T> {
        Self {
            val: [-self.val[0], -self.val[1], -self.val[2], -self.val[3]],
        }
    }

    pub fn conj_mut(&mut self) -> &Quaternion<T> {
        for i in 1..4 {
            self.val[i] = -self.val[i];
        }
        self
    }

    pub fn inv_mut(&mut self) -> &Quaternion<T> {
        for i in 0..4 {
            self.val[i] = -self.val[i];
        }
        self
    }

    pub fn u_mut(mut self) -> Self {
        self = self / self.n();
        self
    }

    pub fn u(self) -> Self {
        Self {
            val: (self / self.n()).val,
        }
    }

    pub fn n(self) -> T {
        (self.val[0] * self.val[0] +
            self.val[1] * self.val[1] +
            self.val[2] * self.val[2] +
            self.val[3] * self.val[3])
            .sqrt()
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut v = Vec::<T>::with_capacity(4);
        v.copy_from_slice(&self.val);
        v
    }

    pub fn sum(&self) -> T {
        self.val[0] + self.val[1] + self.val[2] + self.val[3]
    }

    pub fn dot(
        self,
        b: Quaternion<T>,
    ) -> T {
        self.val
            .iter()
            .zip(b.val.iter())
            .map(|(x, y)| *x * *y)
            .fold(T::zero(), |res, val| res + val)
    }
}

impl<T: Float + From<f32>> From<[T; 3]> for Quaternion<T> {
    fn from(inp: [T; 3]) -> Quaternion<T> { Quaternion::from_slice(&inp) }
}

unsafe impl<T: From<f32> + WhichFloat> VertexMember for Quaternion<T> {
    fn format() -> (VertexMemberTy, usize) { (T::vmt(), T::vms()) }
}

impl<T: Float + From<f32>> One for Quaternion<T> {
    fn one() -> Self { Self { val: [T::one(); 4] } }
}
impl<T: Float + From<f32>> Zero for Quaternion<T> {
    fn zero() -> Self {
        Self {
            val: [T::zero(); 4],
        }
    }

    fn is_zero(&self) -> bool {
        self.val[0] == T::zero() &&
            self.val[1] == T::zero() &&
            self.val[2] == T::zero() &&
            self.val[3] == T::zero()
    }
}

impl<T: Float + From<f32>> Mul<T> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn mul(
        self,
        rhs: T,
    ) -> Quaternion<T> {
        Quaternion {
            val: [
                self.val[0] * rhs,
                self.val[1] * rhs,
                self.val[2] * rhs,
                self.val[3] * rhs,
            ],
        }
    }
}
impl<T: Float + From<f32>> Mul<Quaternion<T>> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn mul(
        self,
        rhs: Quaternion<T>,
    ) -> Quaternion<T> {
        Quaternion {
            val: [
                self.val[0] * rhs.val[0] -
                    self.val[1] * rhs.val[1] -
                    self.val[2] * rhs.val[2] -
                    self.val[3] * rhs.val[3],
                self.val[0] * rhs.val[1] +
                    self.val[1] * rhs.val[0] +
                    self.val[2] * rhs.val[3] -
                    self.val[3] * rhs.val[2],
                self.val[0] * rhs.val[2] - self.val[1] * rhs.val[3] +
                    self.val[2] * rhs.val[0] +
                    self.val[3] * rhs.val[1],
                self.val[0] * rhs.val[3] + self.val[1] * rhs.val[2] -
                    self.val[2] * rhs.val[1] +
                    self.val[3] * rhs.val[0],
            ],
        }
    }
}
impl<T: Float + From<f32>> Mul<Octonion<T>> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn mul(
        self,
        rhs: Octonion<T>,
    ) -> Quaternion<T> {
        self * rhs.q1
    }
}

impl<T: Float + From<f32>> Add<T> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn add(
        self,
        rhs: T,
    ) -> Self {
        Self {
            val: [
                self.val[0] + rhs,
                self.val[1] + rhs,
                self.val[2] + rhs,
                self.val[3] + rhs,
            ],
        }
    }
}
impl<T: Float + From<f32>> Add<Quaternion<T>> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn add(
        self,
        rhs: Quaternion<T>,
    ) -> Self {
        Self {
            val: [
                self.val[0] + rhs.val[0],
                self.val[1] + rhs.val[1],
                self.val[2] + rhs.val[2],
                self.val[3] + rhs.val[3],
            ],
        }
    }
}

impl<T: Float + From<f32>> Sub<T> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn sub(
        self,
        rhs: T,
    ) -> Self {
        Self {
            val: [
                self.val[0] - rhs,
                self.val[1] - rhs,
                self.val[2] - rhs,
                self.val[3] - rhs,
            ],
        }
    }
}
impl<T: Float + From<f32>> Sub<Quaternion<T>> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn sub(
        self,
        rhs: Quaternion<T>,
    ) -> Self {
        Self {
            val: [
                self.val[0] - rhs.val[0],
                self.val[1] - rhs.val[1],
                self.val[2] - rhs.val[2],
                self.val[3] - rhs.val[3],
            ],
        }
    }
}

impl<T: Float + From<f32>> Div<T> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn div(
        self,
        rhs: T,
    ) -> Quaternion<T> {
        Quaternion {
            val: [
                self.val[0] / rhs,
                self.val[1] / rhs,
                self.val[2] / rhs,
                self.val[3] / rhs,
            ],
        }
    }
}
impl<T: Float + From<f32>> Neg for Quaternion<T> {
    type Output = Quaternion<T>;

    fn neg(self) -> Quaternion<T> {
        Quaternion {
            val: [-self.val[0], -self.val[1], -self.val[2], -self.val[3]],
        }
    }
}
// impl<T: Float> Div<Quaternion<T>> for Quaternion<T> {
//     type Output = Quaternion<T>;
//
//     fn div(
//         self,
//         rhs: Quaternion<T>,
//     ) -> Quaternion<T> {
//         let a = self.val[0] / rhs.val[0];
//         let b = self.val[1] / rhs.val[1];
//         let c = self.val[2] / rhs.val[2];
//         let d = self.val[3] / rhs.val[3];
//         Quaternion {
//             val: [a - b - c - d, a + b + c - d, a - b + c + d, a + b - c +
// d],         }
//     }
// }
