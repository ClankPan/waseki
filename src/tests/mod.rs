mod hash;
mod utils;
mod x_op_y;

use crate::var::V;

use super::*;
use ark_bn254::Fr;
use cyclotomic_rings::rings::GoldilocksRingNTT;
use stark_rings::Ring;

#[test]
fn test_linearize() {
    let mut cs = ConstraintSystem::default();
    cs.with_cs::<_, _>(|cs| {
        let a: V<'_, _> = (0..N as u64).map(|n| cs.alloc(Fr::from(n))).sum();
        a.inputize();
        let a = a + cs.alloc(Fr::from(111));
        let a = a + cs.alloc(Fr::from(222)) + cs.alloc(Fr::from(333));
        let a = a + cs.alloc(Fr::from(444));
        // println!("a: {:?}", a);
    });
}

fn demo<R: Ring>() {
    let mut cs = ConstraintSystem::default();
    cs.with_cs::<_, _>(|cs| {
        let l1 = cs.alloc(R::from(1u128));
        let l2 = cs.alloc(R::from(2u128));

        // L + L -> L
        let l = l1 + l2;

        // L * L -> Q
        let q = l * l1;

        // Q + L -> Q
        let q = q + l;

        // Q * L -> Q
        let q = q * l;

        // Q + Q -> Q
        let q = q + q;

        // Q * Q -> Q
        let _q = q * q;

        // // u128 + L -> L;
        // let l = 1u128 + l;
        //
        // // u128 + L -> L;
        // let _l = 1u128 + l;
        //
        // // u128 + Q -> Q;
        // let q = 1u128 + q;
        //
        // // u128 + Q -> Q;
        // let _q = 1u128 + q;
        //
        // let _l = &l + &l;

        // reduce がテープに値を出力しているはず
        // assert!(!cs.view_w().is_empty());
    });
}

#[test]
fn demo_with_goldilocks_ring_ntt() {
    demo::<GoldilocksRingNTT>()
}

use ark_ff::{Field, PrimeField};
use ark_r1cs_std::{
    alloc::AllocVar,
    eq::EqGadget,
    fields::{FieldVar, fp::FpVar},
};
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystem as ArkCS, ConstraintSystemRef,
    SynthesisError,
};
use utils::waseki_pow;

#[test]
fn test_compati_with_arkworks() {
    let matrices = arkworks_pow(2, 3);

    println!("arkworks: {:?}", matrices);
    let r1cs = waseki_pow::<Fr>(2, 3);

    assert!(false)
}

fn arkworks_pow(base: u128, exp: u64) -> ConstraintMatrices<Fr> {
    let cs = ArkCS::<Fr>::new_ref();
    // 例として base と exp, expect を用意
    let base = Fr::from(base);
    // ネイティブ計算した期待値
    let expect = Some(base.pow([exp as u64, 0, 0, 0]));

    // ===== 2) 回路を流し込む（制約生成）=====
    let circuit = Circuit::<Fr> {
        base_val: base,
        exp,
        expect,
    };
    circuit.generate_constraints(cs.clone()).unwrap();

    // ===== 3) 充足性チェック（任意）=====
    // witness を全部与えていれば true になる
    assert!(cs.is_satisfied().unwrap());

    // ===== 4) R1CS を確定させて行列化 =====
    cs.finalize(); // Symbolic LC のインライン化など
    let matrices = cs.to_matrices().unwrap(); // A/B/C のスパース行列とメタ情報
    matrices
}

// use ark_r1cs_std::{alloc::AllocVar, fields::fp::FpVar};

// 例: 回路本体
struct Circuit<Fr: Field> {
    // 好みで公開/秘密どちらでも
    pub base_val: Fr,       // 入力値
    pub exp: u64,           // 定数指数
    pub expect: Option<Fr>, // 検証用（あれば acc == expect を課す）
}

impl<Fr: PrimeField> ConstraintSynthesizer<Fr> for Circuit<Fr> {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // ベースの割り当て（公開入力にしたいなら new_input_variable を使う）
        let base = FpVar::<Fr>::new_input(cs.clone(), || Ok(self.base_val))?;

        // べき乗
        let acc = pow::<Fr>(base, self.exp)?; // FpVar<Fr>

        // 必要なら結果に制約を課す（例: 期待値と等しい）
        if let Some(exp_val) = self.expect {
            let expect_var = FpVar::<Fr>::new_witness(cs, || Ok(exp_val))?;
            acc.enforce_equal(&expect_var)?;
        }

        Ok(())
    }
}

// 二乗法 pow（指数は定数 u64）
fn pow<F: PrimeField>(mut base: FpVar<F>, mut exp: u64) -> Result<FpVar<F>, SynthesisError> {
    let mut acc = FpVar::<F>::constant(F::one());
    while exp > 0 {
        if (exp & 1) == 1 {
            acc = &acc * &base;
        }
        base = &base * &base;
        exp >>= 1;
    }
    Ok(acc)
}
