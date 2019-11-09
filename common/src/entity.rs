use crate::{Entity, Octonion};
use num_traits::{identities::One, Float, Zero};
impl<T: Float> Entity<T> {
    pub fn new() -> Self {
        Self {
            pos: Octonion::<T>::one(),
        }
    }
}
