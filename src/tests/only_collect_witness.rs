use crate::{ConstraintSynthesizer, ConstraintSystem, var::V};
use ark_bn254::Fr;
use ark_ff::UniformRand;

fn vitalic_expample(cs: &ConstraintSynthesizer<Fr>, x: Fr) {
    let x = cs.input(x);

    // x^3 + x + 5 を計算
    let x2 = x * x; // x^2
    let x3 = x2 * x; // x^3
    let five = cs.constant(Fr::from(5u64));
    let expr = x3 + x + five; // x^3 + x + 5
    expr.inputize();
}

#[test]
fn test_only_correct_witness() {
    let mut rng = ark_std::test_rng();
    let mut cs = ConstraintSystem::<Fr>::default();
    cs.synthesize_with(|cs| {
        vitalic_expample(&cs, Fr::rand(&mut rng));
    });
    assert!(cs.is_satisfied());
    cs.synthesize_with(|cs| {
        vitalic_expample(&cs, Fr::rand(&mut rng));
    });
    assert!(cs.is_satisfied());
}

#[test]
fn test_large_linear_quadratic() {
    let mut cs = ConstraintSystem::<Fr>::default();
    cs.synthesize_with(|cs| {
        let x: V<Fr> = (0..10).map(|i| cs.alloc(i)).sum();
        let y: V<Fr> = (0..10).map(|i| cs.alloc(i)).sum();
        let z = x * y;
        let z = z * z;
        z.inputize();
    });
    assert!(cs.is_satisfied());
    dbg!(cs.r1cs);
}
