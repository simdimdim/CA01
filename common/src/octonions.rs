use crate::{Octonion, Quaternion, WhichFloat};
use num_traits::{identities::One, Float, Zero};
use std::ops::{Add, Mul, Sub};
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

impl<T: Float> Octonion<T> {
    pub fn new(vec8: [T; 8]) -> Self {
        Self {
            q1: Quaternion::from_slice(&vec8[0..4]),
            q2: Quaternion::from_slice(&vec8[4..8]),
        }
    }

    pub fn conj(&self) -> Self {
        Self {
            q1: self.q1.conj(),
            q2: self.q2.inv(),
        }
    }

    pub fn conj_mut(&mut self) -> &Self {
        self.q1.conj_mut();
        self.q2.inv_mut();
        self
    }

    pub fn as_array(&self) -> [T; 8] {
        let mut v = [T::one(); 8];
        v[0..4].copy_from_slice(&self.q1.val);
        v[4..8].copy_from_slice(&self.q2.val);
        v
    }

    pub fn lerp(
        self,
        other: Octonion<T>,
        t: T,
    ) -> Octonion<T> {
        self * (Octonion::<T>::one() - t) + other * t
    }

    // pub fn lerp_q(
    //     self,
    //     other: Qaternion<T>,
    //     t: T,
    // ) -> Octonion<T> {
    //     self * (Octonion::<T>::one() - t) + other * t
    // }
}

unsafe impl<T: Float + WhichFloat> VertexMember for Octonion<T> {
    fn format() -> (VertexMemberTy, usize) { (T::vmt(), T::vms()) }
}

impl<T: Float> One for Octonion<T> {
    fn one() -> Self {
        Self {
            q1: Quaternion::one(),
            q2: Quaternion::one(),
        }
    }
}
impl<T: Float> Zero for Octonion<T> {
    fn zero() -> Self {
        Self {
            q1: Quaternion::zero(),
            q2: Quaternion::zero(),
        }
    }

    fn is_zero(&self) -> bool { self.q1.is_zero() && self.q2.is_zero() }
}

impl<T: Float> From<Quaternion<T>> for Octonion<T> {
    fn from(q: Quaternion<T>) -> Octonion<T> {
        Self {
            q1: q,
            q2: Quaternion::<T>::zero(),
        }
    }
}
impl<T: Float> From<(Quaternion<T>, Quaternion<T>)> for Octonion<T> {
    fn from(q: (Quaternion<T>, Quaternion<T>)) -> Octonion<T> {
        Self { q1: q.0, q2: q.1 }
    }
}

impl<T: Float> Mul<T> for Octonion<T> {
    type Output = Octonion<T>;

    fn mul(
        self,
        rhs: T,
    ) -> Octonion<T> {
        Octonion {
            q1: self.q1 * rhs,
            q2: self.q2 * rhs,
        }
    }
}
impl<T: Float> Mul<Octonion<T>> for Octonion<T> {
    type Output = Octonion<T>;

    fn mul(
        self,
        rhs: Octonion<T>,
    ) -> Octonion<T> {
        Octonion {
            q1: self.q1 * rhs.q1 - rhs.q2.conj() * self.q2,
            q2: rhs.q2 * self.q1 + self.q2 * rhs.q1.conj(),
        }
    }
}
impl<T: Float> Mul<Quaternion<T>> for Octonion<T> {
    type Output = Octonion<T>;

    fn mul(
        self,
        rhs: Quaternion<T>,
    ) -> Octonion<T> {
        self * Octonion::<T>::from(rhs)
    }
}

impl<T: Float> Add<T> for Octonion<T> {
    type Output = Octonion<T>;

    fn add(
        self,
        rhs: T,
    ) -> Octonion<T> {
        Octonion {
            q1: self.q1 + rhs,
            q2: self.q2 + rhs,
        }
    }
}
impl<T: Float> Add<Octonion<T>> for Octonion<T> {
    type Output = Octonion<T>;

    fn add(
        self,
        rhs: Octonion<T>,
    ) -> Octonion<T> {
        Self {
            q1: self.q1 + rhs.q1,
            q2: self.q2 + rhs.q2,
        }
    }
}

impl<T: Float> Sub<T> for Octonion<T> {
    type Output = Octonion<T>;

    fn sub(
        self,
        rhs: T,
    ) -> Octonion<T> {
        Octonion {
            q1: self.q1 - rhs,
            q2: self.q2 - rhs,
        }
    }
}
impl<T: Float> Sub<Octonion<T>> for Octonion<T> {
    type Output = Octonion<T>;

    fn sub(
        self,
        rhs: Octonion<T>,
    ) -> Octonion<T> {
        Self {
            q1: self.q1 - rhs.q1,
            q2: self.q2 - rhs.q2,
        }
    }
}
