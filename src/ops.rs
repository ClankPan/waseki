use ark_ff::Field;
use num_traits::One;
use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::{
    list::List,
    state::{self, Index, with_state},
    var::Var,
};

impl<F: Field> Add for Var<F> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.value += rhs.value;
        self.stateful &= rhs.stateful;
        if self.stateful {
            for entry in rhs.list.list {
                let Some((coeff, index)) = entry else { break };
                self.list.push(coeff, index);
            }
        }
        self
    }
}

impl<F: Field> AddAssign for Var<F> {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
        self.stateful &= rhs.stateful;
        if self.stateful {
            for entry in rhs.list.list {
                let Some((coeff, index)) = entry else { break };
                self.list.push(coeff, index);
            }
        }
    }
}

impl<F: Field> Sub for Var<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs * F::one().neg()
    }
}

impl<F: Field> SubAssign for Var<F> {
    fn sub_assign(&mut self, rhs: Self) {
        *self += rhs * F::one().neg();
    }
}

impl<F: Field> Add<F> for Var<F> {
    type Output = Self;

    fn add(mut self, rhs: F) -> Self::Output {
        self.value += rhs;
        if self.stateful {
            self.list.push(rhs, Index::I(0));
        }
        self
    }
}

impl<F: Field> AddAssign<F> for Var<F> {
    fn add_assign(&mut self, rhs: F) {
        self.value += rhs;
        if self.stateful {
            self.list.push(rhs, Index::I(0));
        }
    }
}

impl<F: Field> SubAssign<F> for Var<F> {
    fn sub_assign(&mut self, rhs: F) {
        *self += rhs.neg();
    }
}

impl<F: Field> Mul for Var<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let value = self.value * rhs.value;
        if self.stateful && rhs.stateful {
            if let Some(index) = state::alloc(&value) {
                if let Some(_) = with_state(|state| {
                    let a_idx = state.push_linear_list(&self.list);
                    let b_idx = state.push_linear_list(&rhs.list);
                    let a = (a_idx, state::serialize_value(&self.value));
                    let b = (b_idx, state::serialize_value(&rhs.value));
                    let c = (index, state::serialize_value(&value));
                    state.push_quadratic_lists(a, b, c);
                }) {
                    return Self {
                        value,
                        list: List::new(index),
                        stateful: true,
                    };
                }
            }
        }
        Self {
            value,
            list: List::empty(),
            stateful: false,
        }
    }
}

impl<F: Field> MulAssign for Var<F> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<F: Field> Mul<F> for Var<F> {
    type Output = Self;

    fn mul(mut self, rhs: F) -> Self::Output {
        self.value *= rhs;
        if self.stateful {
            self.list.apply(rhs);
        }
        self
    }
}

impl<F: Field> MulAssign<F> for Var<F> {
    fn mul_assign(&mut self, rhs: F) {
        self.value *= rhs;
        if self.stateful {
            self.list.apply(rhs);
        }
    }
}

impl<F: Field> Sum for Var<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Var::from(F::zero()), |acc, x| acc + x)
    }
}

impl<F: Field> Product for Var<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, x| acc * x)
    }
}
