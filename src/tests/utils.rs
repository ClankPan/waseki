use ark_ff::Field;

use crate::{ConstraintSynthesizer, ConstraintSystem, var::V};
type CS<'a, F> = ConstraintSynthesizer<'a, F>;

pub fn pow<'a, F: Field>(cs: CS<'a, F>, mut base: V<'a, F>, mut exp: u64) -> V<'a, F> {
    let mut pow = cs.one();
    while exp > 0 {
        if exp % 2 == 1 {
            pow = pow * base;
        }
        base = base * base;
        exp /= 2;
    }

    pow
}

#[test]
pub fn test_pow() {
    use ark_bn254::Fr;
    let mut cs = ConstraintSystem::default();
    cs.with_cs(|cs| {
        let a = cs.alloc(Fr::from(2));
        let b = pow(cs, a, 3);

        assert_eq!(b.value(), Fr::from(8));
    });
}
