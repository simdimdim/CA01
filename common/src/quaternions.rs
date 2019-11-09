use crate::Quaternion;
use num_traits::{Float, One, Zero};
use std::ops::{Add, Mul, Sub};

impl<T: Float> Quaternion<T> {
    pub fn new(i: [T; 4]) -> Self { Self { val: i } }

    pub fn from_slice(inp: &[T]) -> Self {
        let mut q = Quaternion::zero();
        q.val.copy_from_slice(inp);
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
impl<T: Float> Mul<T> for Quaternion<T> {
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
impl<T: Float> Mul<Quaternion<T>> for Quaternion<T> {
    type Output = Quaternion<T>;

    fn mul(
        self,
        rhs: Quaternion<T>,
    ) -> Quaternion<T> {
        let a = self.val[0] * rhs.val[0];
        let b = self.val[1] * rhs.val[1];
        let c = self.val[2] * rhs.val[2];
        let d = self.val[3] * rhs.val[3];
        Quaternion {
            val: [a - b - c - d, a + b + c - d, a - b + c + d, a + b - c + d],
        }
    }
}

impl<T: Float> Add<T> for Quaternion<T> {
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
impl<T: Float> Add<Quaternion<T>> for Quaternion<T> {
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

impl<T: Float> Sub<T> for Quaternion<T> {
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
impl<T: Float> Sub<Quaternion<T>> for Quaternion<T> {
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

impl<T: Float> One for Quaternion<T> {
    fn one() -> Self { Self { val: [T::one(); 4] } }
}
impl<T: Float> Zero for Quaternion<T> {
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
