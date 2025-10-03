use crate::{CS, L, var::V};
use ark_ff::Field;

pub fn pow<'a, F: Field>(cs: CS<'a, F>, mut base: L<'a, F>, mut exp: u64) -> L<'a, F> {
    let mut pow = cs.one();
    while exp > 0 {
        if exp % 2 == 1 {
            pow = (pow * base).reduce();
        }
        base = (base * base).reduce();
        exp /= 2;
    }

    pow.into()
}

pub fn pow_v<'a, F: Field>(cs: CS<'a, F>, mut base: V<'a, F>, mut exp: u64) -> V<'a, F> {
    let mut pow = V::L(cs.one());
    while exp > 0 {
        if exp % 2 == 1 {
            pow = pow * base;
        }
        base = base * base;
        exp /= 2;
    }

    pow
}
