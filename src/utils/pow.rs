use ark_ff::PrimeField;
use num_traits::One;

use crate::Var;

impl<F: PrimeField> Var<F> {
    pub fn pow(mut self, mut exp: u64) -> Self {
        let mut pow = Self::one();
        while exp > 0 {
            if exp % 2 == 1 {
                pow *= self;
            }
            self = self * self;
            exp /= 2;
        }

        pow
    }
}
