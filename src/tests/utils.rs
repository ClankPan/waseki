use crate::{CS, L};
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
