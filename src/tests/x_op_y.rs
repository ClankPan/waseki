use ark_bn254::Fr;

use crate::{ConstraintSystem, L, N, l_add_l};

#[test]
fn test_l_add_l() {
    let mut cs = ConstraintSystem::default();
    cs.synthesize_with(|cs| {
        let v_a = Fr::from(111);
        let v_b = Fr::from(222);
        let l_a = L::constant(cs.ar, v_a);
        let l_b = L::constant(cs.ar, v_b);
        let l_c = l_add_l(l_a, l_b);
        assert_eq!(l_a.l.to_vec(), vec![(0, v_a)]);
        assert_eq!(l_b.l.to_vec(), vec![(0, v_b)]);
        assert_eq!(l_c.l.to_vec(), vec![(0, v_a), (0, v_b)]);

        let l_a: L<'_, _> = (0..N)
            .map(|_| l_a)
            .fold(L::new(cs.ar), |acc, x| l_add_l(acc, x));
        let l_c = l_add_l(l_a, l_a);
        assert_eq!(l_c.l.to_vec(), vec![(1, Fr::from(1))]);

        assert_eq!(
            cs.ar.auxes.borrow()[1],
            (0..N).map(|_| v_a).sum::<Fr>() * Fr::from(2)
        );

        let l_c = l_add_l(l_a, l_a);
        assert_eq!(l_c.l.to_vec(), vec![(2, Fr::from(1))]);
    })
}
