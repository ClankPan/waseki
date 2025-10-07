use ark_ff::Field;

use crate::state::{self, Index, N, with_state};

#[derive(Copy, Clone, Debug)]
pub struct List<F: Field> {
    pub(crate) list: [Option<(F, Index)>; N],
    pub(crate) len: usize,
}

impl<F: Field> List<F> {
    pub fn empty() -> Self {
        Self {
            list: [None; N],
            len: 0,
        }
    }

    pub fn new(index: Index) -> Self {
        let mut list = Self::empty();
        list.push(F::one(), index);
        list
    }

    pub fn push(&mut self, coeff: F, index: Index) {
        self.list[self.len] = Some((coeff, index));
        self.len += 1;
        if self.len == N {
            if let Some(new_index) =
                with_state(|state| state.push_linear_entries(self.serialized()))
            {
                *self = Self::new(new_index);
            }
        }
    }

    pub fn apply(&mut self, coeff: F) {
        self.list.iter_mut().for_each(|entry| {
            if let Some((coeff_ref, _)) = entry {
                *coeff_ref *= coeff;
            }
        });
    }

    pub fn serialized(&self) -> Vec<(Vec<u8>, Index)> {
        self.list
            .into_iter()
            .filter_map(state::serialize)
            .collect()
    }

    pub fn terms(&self) -> Vec<(F, Index)> {
        self.list
            .iter()
            .take(self.len)
            .filter_map(|entry| entry.as_ref().copied())
            .collect()
    }
}
