use ark_ff::Field;
use num_traits::One;
use std::fmt::{self, Display};

use crate::{
    list::List,
    state::{
        self, Index, LocalState, SparseRow, deserialize_field, has_state, init_local_state,
        serialize_value, take_local_state, with_state,
    },
};

pub struct CompiledR1CS<F: Field> {
    pub inputs: Vec<F>,
    pub witness: Vec<F>,
    pub a: Vec<SparseRow<F>>,
    pub b: Vec<SparseRow<F>>,
    pub c: Vec<SparseRow<F>>,
    pub lc: (Vec<F>, Vec<F>, Vec<F>),
}

impl<F: Field> CompiledR1CS<F> {
    pub fn assignment(&self) -> Vec<F> {
        let mut assignment = self.inputs.clone();
        assignment.extend_from_slice(&self.witness);
        assignment
    }

    pub fn is_satisfied(&self) -> bool {
        let assignment = self.assignment();
        self.a
            .iter()
            .zip(&self.b)
            .zip(&self.c)
            .all(|((a_row, b_row), c_row)| {
                eval_row(a_row, &assignment) * eval_row(b_row, &assignment)
                    == eval_row(c_row, &assignment)
            })
    }
}

impl<F: Field> Display for CompiledR1CS<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "R1CS rows: {}", self.a.len())?;
        for (i, ((a_row, b_row), c_row)) in self.a.iter().zip(&self.b).zip(&self.c).enumerate() {
            writeln!(f, "Row {}:", i)?;
            writeln!(f, "  A: {} -> {:?}", self.lc.0[i], a_row)?;
            writeln!(f, "  B: {} -> {:?}", self.lc.1[i], b_row)?;
            writeln!(f, "  C: {} -> {:?}", self.lc.2[i], c_row)?;
        }
        Ok(())
    }
}

fn eval_row<F: Field>(row: &SparseRow<F>, assignment: &[F]) -> F {
    row.iter().fold(F::zero(), |acc, (col, coeff)| {
        acc + *coeff * assignment[*col]
    })
}

#[derive(Clone, Copy)]
pub struct Var<F: Field> {
    pub(crate) list: List<F>,
    pub(crate) value: F,
    pub(crate) stateful: bool,
}

impl<F: Field> Var<F> {
    pub fn from(value: F) -> Self {
        if has_state() {
            let index = state::alloc(&value).expect("state missing despite has_state");
            let list = List::new(index);
            Self {
                list,
                value,
                stateful: true,
            }
        } else {
            Self {
                list: List::empty(),
                value,
                stateful: false,
            }
        }
    }

    pub fn value(&self) -> F {
        self.value
    }

    pub fn linear_terms(&self) -> Vec<(F, Index)> {
        self.list.terms()
    }

    pub fn equal(&self, rhs: &Self) {
        if self.stateful && rhs.stateful {
            if let Some(_) = with_state(|state| {
                let a_idx = state.push_linear_list(&self.list);
                let c_idx = state.push_linear_list(&rhs.list);
                let a = (a_idx, serialize_value(&self.value));
                let b = (Index::I(0), serialize_value(&F::one()));
                let c = (c_idx, serialize_value(&rhs.value));
                state.push_quadratic_lists(a, b, c);
            }) {}
        }
    }
}

impl<F: Field> One for Var<F> {
    fn one() -> Self {
        if has_state() {
            Self {
                list: List::new(Index::I(0)),
                value: F::one(),
                stateful: true,
            }
        } else {
            Self {
                list: List::empty(),
                value: F::one(),
                stateful: false,
            }
        }
    }

    fn is_one(&self) -> bool {
        self.value == F::one()
    }
}

pub struct ConstraintSystem<F: Field> {
    _marker: std::marker::PhantomData<F>,
    input: Vec<F>,
    consumed: bool,
}

impl<F: Field> ConstraintSystem<F> {
    pub fn new() -> Self {
        init_local_state();
        Self {
            _marker: std::marker::PhantomData,
            input: vec![F::one()],
            consumed: false,
        }
    }

    pub fn input(&mut self, value: F) -> Var<F> {
        let index = self.input.len();
        self.input.push(value);
        Var {
            list: List::new(Index::I(index)),
            value,
            stateful: true,
        }
    }

    pub fn inputize(&mut self, var: Var<F>) {
        let index = self.input.len();
        self.input.push(var.value);
        let c_idx = Index::I(index);

        with_state(|state| {
            let a_idx = state.push_linear_list(&var.list);
            let a = (a_idx, serialize_value(&var.value));
            let b = (Index::I(0), serialize_value(&F::one()));
            let c = (c_idx, serialize_value(&var.value));
            state.push_quadratic_lists(a, b, c);
        })
        .expect("constraint system state should be initialized");
    }

    pub fn into_state(mut self) -> LocalState {
        self.consumed = true;
        take_local_state().expect("LocalState should exist when consuming ConstraintSystem")
    }

    pub fn compile(self) -> CompiledR1CS<F> {
        let inputs = self.input.clone();
        let state = self.into_state();
        let LocalState {
            witness,
            linear,
            quadratic,
            ..
        } = state;

        let witness: Vec<F> = witness.iter().map(deserialize_field::<F>).collect();
        let input_len = inputs.len();
        let mut cache: Vec<Option<SparseRow<F>>> = vec![None; linear.len()];

        let mut expand = |idx: Index| -> SparseRow<F> {
            crate::state::expand_index(idx, input_len, &linear, &mut cache)
        };

        let mut a = Vec::with_capacity(quadratic.len());
        let mut b = Vec::with_capacity(quadratic.len());
        let mut c = Vec::with_capacity(quadratic.len());

        let mut lc_a = Vec::with_capacity(quadratic.len());
        let mut lc_b = Vec::with_capacity(quadratic.len());
        let mut lc_c = Vec::with_capacity(quadratic.len());

        for ((a_idx, a_bytes), (b_idx, b_bytes), (c_idx, c_bytes)) in quadratic {
            let expanded_a_raw = expand(a_idx);
            let expanded_b_raw = expand(b_idx);
            let expanded_c_raw = expand(c_idx);
            let a_value = deserialize_field(&a_bytes);
            let b_value = deserialize_field(&b_bytes);
            let c_value = deserialize_field(&c_bytes);
            lc_a.push(a_value);
            lc_b.push(b_value);
            lc_c.push(c_value);
            a.push(expanded_a_raw);
            b.push(expanded_b_raw);
            c.push(expanded_c_raw);
        }

        CompiledR1CS {
            inputs,
            witness,
            a,
            b,
            c,
            lc: (lc_a, lc_b, lc_c),
        }
    }
}

impl<F: Field> Drop for ConstraintSystem<F> {
    fn drop(&mut self) {
        if !self.consumed {
            let _ = take_local_state();
        }
    }
}
