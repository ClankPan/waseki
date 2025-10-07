use ark_bn254::Fr;
use num_traits::One;

use waseki::{ConstraintSystem, Index, LocalState, Var, N, init_local_state, take_local_state};

fn setup() {
    init_local_state();
}

fn teardown() -> LocalState {
    take_local_state().expect("local state should be initialized")
}

#[test]
fn var_from_records_witness() {
    setup();
    let a = Var::from(Fr::from(5u64));
    assert_eq!(a.value(), Fr::from(5u64));
    let state = teardown();
    assert_eq!(state.witness.len(), 1);
    assert!(state.linear.is_empty());
    assert!(state.quadratic.is_empty());
}

#[test]
fn var_one_points_to_constant_input() {
    setup();
    let one = Var::<Fr>::one();
    assert_eq!(one.value(), Fr::one());
    let terms = one.linear_terms();
    assert_eq!(terms.len(), 1);
    assert!(matches!(terms[0].1, Index::I(0)));
    teardown();
}

#[test]
fn scalar_add_uses_constant_input() {
    setup();
    let var = Var::from(Fr::from(10u64));
    let _ = var + Fr::from(7u64);
    let state = teardown();
    assert_eq!(state.witness.len(), 1);
    assert!(state.linear.is_empty());
    assert!(state.quadratic.is_empty());
}

#[test]
fn multiplication_adds_quadratic_constraint() {
    setup();
    let a = Var::from(Fr::from(3u64));
    let b = Var::from(Fr::from(4u64));
    let product = a * b;
    assert_eq!(product.value(), Fr::from(12u64));
    let output_index = product.linear_terms()[0].1;
    let state = teardown();
    assert_eq!(state.witness.len(), 3);
    assert_eq!(state.quadratic.len(), 1);
    assert_eq!(state.linear.len(), 2);
    assert!(matches!(state.quadratic[0].0 .0, Index::L(0)));
    assert!(matches!(state.quadratic[0].1 .0, Index::L(1)));
    assert_eq!(state.quadratic[0].2 .0, output_index);
}

#[test]
fn equality_pushes_linear_constraint() {
    setup();
    let a = Var::from(Fr::from(5u64));
    let b = Var::from(Fr::from(5u64));
    a.equal(&b);
    let state = teardown();
    assert_eq!(state.linear.len(), 2);
    assert_eq!(state.quadratic.len(), 1);
    assert!(matches!(state.quadratic[0].0 .0, Index::L(0)));
    assert!(matches!(state.quadratic[0].2 .0, Index::L(1)));
}

#[test]
fn long_linear_combination_is_spilled() {
    setup();
    let mut acc = Var::from(Fr::from(0u64));
    for i in 0..=N {
        let rhs = Var::from(Fr::from(i as u64 + 1));
        acc = acc + rhs;
    }
    let state = teardown();
    assert!(!state.linear.is_empty());
}

#[test]
fn constraint_system_scopes_state() {
    {
        let _cs = ConstraintSystem::<Fr>::new();
        let _ = Var::from(Fr::from(1u64));
    }
    assert!(take_local_state().is_none());
}

#[test]
fn operations_without_state_hold_value_only() {
    assert!(take_local_state().is_none());
    let a = Var::from(Fr::from(2u64));
    let b = Var::from(Fr::from(3u64));
    let sum = a + b;
    assert_eq!(sum.value(), Fr::from(5u64));
    assert!(sum.linear_terms().is_empty());
    let product = a * b;
    assert_eq!(product.value(), Fr::from(6u64));
    assert!(product.linear_terms().is_empty());
}

#[test]
fn consume_constraint_system_returns_state() {
    let cs = ConstraintSystem::<Fr>::new();
    let a = Var::from(Fr::from(2u64));
    let b = Var::from(Fr::from(3u64));
    let _ = a * b;
    let state = cs.into_state();
    assert_eq!(state.witness.len(), 3);
    assert_eq!(state.quadratic.len(), 1);
    assert!(take_local_state().is_none());
}

#[test]
fn compile_produces_valid_r1cs() {
    let mut cs = ConstraintSystem::<Fr>::new();
    let x = cs.input(Fr::from(3u64));
    let y = cs.input(Fr::from(4u64));
    let prod = x * y;
    cs.inputize(prod);
    let compiled = cs.compile();

    assert_eq!(compiled.inputs.len(), 4);
    assert_eq!(compiled.witness.len(), 1);
    assert_eq!(compiled.a.len(), compiled.b.len());
    assert_eq!(compiled.a.len(), compiled.c.len());
    assert_eq!(compiled.constraints.len(), compiled.a.len());
    for i in 0..compiled.constraints.len() {
        let (a_vals, b_vals, c_vals) = &compiled.constraints[i];
        assert_eq!(a_vals, &compiled.a[i]);
        assert_eq!(b_vals, &compiled.b[i]);
        assert_eq!(c_vals, &compiled.c[i]);
    }
    assert!(compiled.is_satisfied());
}

#[test]
fn complex_constraint_structure() {
    let mut cs = ConstraintSystem::<Fr>::new();
    let x = cs.input(Fr::from(3u64));
    let y = cs.input(Fr::from(4u64));
    let z = cs.input(Fr::from(5u64));

    let sum1 = x + y;
    cs.inputize(sum1);
    let sum2 = y + z;
    cs.inputize(sum2);
    let prod = sum1 * sum2;
    cs.inputize(prod);

    let compiled = cs.compile();
    assert_eq!(compiled.a.len(), 4);
    assert_eq!(compiled.constraints.len(), compiled.a.len());
    assert_eq!(compiled.constraints.len(), compiled.a.len());
    assert!(compiled.is_satisfied());

    assert_eq!(compiled.a[0], vec![(1, Fr::one()), (2, Fr::one())]);
    assert_eq!(compiled.b[0], vec![(0, Fr::one())]);
    assert_eq!(compiled.c[0], vec![(4, Fr::one())]);

    assert_eq!(compiled.a[1], vec![(2, Fr::one()), (3, Fr::one())]);
    assert_eq!(compiled.b[1], vec![(0, Fr::one())]);
    assert_eq!(compiled.c[1], vec![(5, Fr::one())]);

    assert_eq!(compiled.a[2], vec![(1, Fr::one()), (2, Fr::one())]);
    assert_eq!(compiled.b[2], vec![(2, Fr::one()), (3, Fr::one())]);
    assert_eq!(compiled.c[2], vec![(7, Fr::one())]);

    assert_eq!(compiled.a[3], vec![(7, Fr::one())]);
    assert_eq!(compiled.b[3], vec![(0, Fr::one())]);
    assert_eq!(compiled.c[3], vec![(6, Fr::one())]);

    for i in 0..compiled.constraints.len() {
        let (a_vals, b_vals, c_vals) = &compiled.constraints[i];
        assert_eq!(a_vals, &compiled.a[i]);
        assert_eq!(b_vals, &compiled.b[i]);
        assert_eq!(c_vals, &compiled.c[i]);
    }
}
