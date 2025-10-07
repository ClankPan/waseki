use std::{cell::RefCell, collections::BTreeMap};

use ark_ff::Field;

pub const N: usize = 64;

pub type Bytes = Vec<u8>;

#[derive(Debug, Default)]
pub struct LocalState {
    pub witness: Vec<Bytes>,
    pub linear: Vec<Vec<(Bytes, Index)>>,
    pub quadratic: Vec<((Index, Bytes), (Index, Bytes), (Index, Bytes))>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Index {
    I(usize),
    W(usize),
    L(usize),
}

pub type SparseRow<F> = Vec<(usize, F)>;

thread_local! {
    static LOCAL_STATE: RefCell<Option<LocalState>> = const { RefCell::new(None) };
}

pub fn init_local_state() {
    LOCAL_STATE.with(|state| {
        let mut state_ref = state.borrow_mut();
        if state_ref.is_some() {
            panic!("LocalState already initialized");
        }
        *state_ref = Some(LocalState::default());
    });
}

pub fn take_local_state() -> Option<LocalState> {
    LOCAL_STATE.with(|state| state.borrow_mut().take())
}

pub fn with_state<R>(f: impl FnOnce(&mut LocalState) -> R) -> Option<R> {
    LOCAL_STATE.with(|state| {
        let mut state = state.borrow_mut();
        let state = state.as_mut()?;
        Some(f(state))
    })
}

pub fn has_state() -> bool {
    LOCAL_STATE.with(|state| state.borrow().is_some())
}

pub fn alloc<F: Field>(value: &F) -> Option<Index> {
    with_state(|state| {
        let bytes = serialize_value(value);
        let index = state.witness.len();
        state.witness.push(bytes);
        Index::W(index)
    })
}

pub fn serialize<F: Field>(a: Option<(F, Index)>) -> Option<(Bytes, Index)> {
    let (coeff, index) = a?;
    Some((serialize_value(&coeff), index))
}

pub fn serialize_value<F: Field>(value: &F) -> Bytes {
    let mut bytes = Vec::new();
    value.serialize_compressed(&mut bytes).unwrap();
    bytes
}

pub fn deserialize_field<F: Field>(bytes: &Bytes) -> F {
    F::deserialize_compressed(bytes.as_slice()).expect("failed to deserialize field element")
}

impl LocalState {
    pub fn push_linear_entries(&mut self, entries: Vec<(Bytes, Index)>) -> Index {
        let index = self.linear.len();
        self.linear.push(entries);
        Index::L(index)
    }

    pub fn push_linear_list<F: Field>(&mut self, list: &crate::list::List<F>) -> Index {
        let entries = list.serialized();
        self.push_linear_entries(entries)
    }

    pub fn push_quadratic_lists(
        &mut self,
        a: (Index, Bytes),
        b: (Index, Bytes),
        c: (Index, Bytes),
    ) {
        self.quadratic.push((a, b, c));
    }

}

pub fn expand_index<F: Field>(
    index: Index,
    input_len: usize,
    linear: &[Vec<(Bytes, Index)>],
    cache: &mut [Option<SparseRow<F>>],
) -> SparseRow<F> {
    match index {
        Index::I(i) => vec![(i, F::one())],
        Index::W(i) => vec![(input_len + i, F::one())],
        Index::L(i) => expand_linear(i, input_len, linear, cache),
    }
}

fn expand_linear<F: Field>(
    linear_index: usize,
    input_len: usize,
    linear: &[Vec<(Bytes, Index)>],
    cache: &mut [Option<SparseRow<F>>],
) -> SparseRow<F> {
    if let Some(row) = &cache[linear_index] {
        return row.clone();
    }

    let mut acc = BTreeMap::new();
    for (bytes, idx) in &linear[linear_index] {
        let coeff = deserialize_field::<F>(bytes);
        let inner = expand_index(*idx, input_len, linear, cache);
        for (col, value) in inner {
            let entry = coeff * value;
            acc.entry(col)
                .and_modify(|current| *current += entry)
                .or_insert(entry);
        }
    }

    let row: SparseRow<F> = acc.into_iter().collect();
    cache[linear_index] = Some(row.clone());
    row
}
