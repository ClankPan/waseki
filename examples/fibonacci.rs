use ark_bn254::Fr;
use num_traits::One;
use waseki::ConstraintSystem;

fn main() {
    let mut cs = ConstraintSystem::<Fr>::new();

    let f0 = cs.input(Fr::one());
    let f1 = cs.input(Fr::one());
    let f2 = f0 + f1;
    cs.inputize(f2);

    let f3 = f1 + f2;
    cs.inputize(f3);

    let compiled = cs.compile();
    assert!(compiled.is_satisfied());
    println!("{}", compiled);
}
