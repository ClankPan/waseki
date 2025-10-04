mod hash;
mod utils;

use super::*;
use cyclotomic_rings::rings::GoldilocksRingNTT;
use stark_rings::Ring;

fn demo<R: Ring>() {
    with_cs::<R, _, _>(|cs| {
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
        let q = q * q;

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
