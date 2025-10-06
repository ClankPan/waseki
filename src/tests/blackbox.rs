use ark_bn254::Fr;
use ark_ff::{Field, PrimeField};
use ark_r1cs_std::{
    alloc::AllocVar,
    eq::EqGadget,
    fields::{FieldVar, fp::FpVar},
};
use ark_relations::r1cs::{
    ConstraintSynthesizer as ArkConstraintSynthesizer, ConstraintSystem as ArkConstraintSystem,
    ConstraintSystemRef as ArkConstraintSystemRef, SynthesisError,
};

use crate::{ConstraintSystem as WasekiConstraintSystem, r1cs::R1CS};

/// x^3 + x + 5 = y を証明する回路
#[derive(Clone, Debug)]
pub struct Circuit<Fr: PrimeField> {
    /// 秘密入力（witness）
    pub x: Fr,
    /// 公開入力（public input）
    pub y: Fr,
}

impl<Fr: PrimeField> ArkConstraintSynthesizer<Fr> for Circuit<Fr> {
    fn generate_constraints(self, cs: ArkConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // 公開入力 y
        let y = FpVar::<Fr>::new_input(cs.clone(), || Ok(self.y))?;

        // 秘密入力 x
        let x = FpVar::<Fr>::new_witness(cs.clone(), || Ok(self.x))?;

        // x^3 + x + 5 を計算
        let x2 = &x * &x; // x^2
        let x3 = &x2 * &x; // x^3
        let five = FpVar::<Fr>::constant(Fr::from(5u64));
        let expr = x3 + &x + five; // x^3 + x + 5

        // 制約: expr == y
        expr.enforce_equal(&y)?;

        Ok(())
    }
}

fn waseki_circuit(x: Fr, y: Fr) -> R1CS<Fr> {
    let mut cs = WasekiConstraintSystem::<Fr>::default();
    cs.with_cs(|cs| {
        let x = cs.input(x);
        let y = cs.input(y);

        // x^3 + x + 5 を計算
        let x2 = x * x; // x^2
        let x3 = x2 * x; // x^3
        x3.inputize();
        // let five = cs.constant(Fr::from(5u64));
        // let expr = x3 + x + five; // x^3 + x + 5
        //
        // // 制約: expr == y
        // expr.equals(y);
    });
    let r1cs = cs.r1cs.unwrap();
    println!("waseki:\n{:?}\n", r1cs);
    println!("witness: {:?}", cs.witness);
    r1cs
}

#[test]
fn test_arkworks_cimpatibility() {
    // 例: x = 3 のとき y = 3^3 + 3 + 5 = 35
    let x_val = Fr::from(3u64);
    let mut y_val = x_val; // 3
    y_val *= x_val; // 9
    y_val *= x_val; // 27
    y_val += x_val; // 30
    y_val += Fr::from(5u64); // 35

    let cs = ArkConstraintSystem::<Fr>::new_ref();
    let circuit = Circuit::<Fr> { x: x_val, y: y_val };

    circuit.generate_constraints(cs.clone()).unwrap();
    cs.finalize(); // Symbolic LC のインライン化など

    assert!(cs.is_satisfied().unwrap(), "constraints not satisfied");

    let matrices = cs.to_matrices().unwrap(); // A/B/C のスパース行列とメタ情報
    println!("arkworks:\n{:?}\n", matrices);
    waseki_circuit(x_val, y_val);
}
