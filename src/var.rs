use num_traits::{One, Zero};
use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul},
};

use crate::{
    L, Q, ar::Arena, l_add_l, l_mul_l, q_add_l, q_add_q, q_mul_l, q_mul_q, t_add_l, t_add_q,
    t_mul_l, t_mul_q,
};

#[derive(Copy, Clone, Debug)]
pub enum V<'id, T> {
    N,
    L(L<'id, T>),
    Q(Q<'id, T>),
}

impl<'id, T> V<'id, T>
where
    T: One + Zero + Copy,
{
    pub fn new(ar: &'id Arena<T>) -> Self {
        // Self::L(L::new(ar))
        Self::N
    }
    pub fn value(&self) -> T {
        match self {
            V::L(l) => l.value(),
            V::Q(q) => q.value(),
            V::N => T::zero(),
        }
    }
}

// V + V -> V
impl<'id, T> Add for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (V::L(x), V::L(y)) => V::L(l_add_l(x, y)),
            (V::L(l), V::Q(q)) => V::Q(q_add_l(q, l)),
            (V::Q(q), V::L(l)) => V::Q(q_add_l(q, l)),
            (V::Q(x), V::Q(y)) => V::Q(q_add_q(x, y)),
            (V::N, V::N) => V::N,
            (V::N, V::L(l)) => V::L(l),
            (V::N, V::Q(q)) => V::Q(q),
            (V::L(l), V::N) => V::L(l),
            (V::Q(q), V::N) => V::Q(q),
        }
    }
}

// V * V -> V
impl<'id, T> Mul for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (V::L(x), V::L(y)) => V::Q(l_mul_l(x, y)),
            (V::L(l), V::Q(q)) => V::Q(q_mul_l(q, l)),
            (V::Q(q), V::L(l)) => V::Q(q_mul_l(q, l)),
            (V::Q(x), V::Q(y)) => V::Q(q_mul_q(x, y)),
            (V::N, V::N) => V::N,
            (V::N, V::L(l)) => V::L(l),
            (V::N, V::Q(q)) => V::Q(q),
            (V::L(l), V::N) => V::L(l),
            (V::Q(q), V::N) => V::Q(q),
        }
    }
}

// &V + &V -> V
impl<'id, T> Add for &V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        *self + *rhs
    }
}

// &V * &V -> V
impl<'id, T> Mul for &V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        *self * *rhs
    }
}

// V + T -> V
impl<'id, T> Add<T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn add(self, rhs: T) -> Self::Output {
        match self {
            V::L(l) => V::L(t_add_l(rhs, l)),
            V::Q(q) => V::Q(t_add_q(rhs, q)),
            V::N => V::N,
        }
    }
}

// V * T -> V
impl<'id, T> Mul<T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn mul(self, rhs: T) -> Self::Output {
        match self {
            V::L(l) => V::L(t_mul_l(rhs, l)),
            V::Q(q) => V::Q(t_mul_q(rhs, q)),
            V::N => V::N,
        }
    }
}

// V += V
impl<'id, T> AddAssign for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = &*self + &rhs;
    }
}

// V += &V
impl<'id, T> AddAssign<&V<'id, T>> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        *self = &*self + rhs;
    }
}

// V += T
impl<'id, T> AddAssign<T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

// V += &T
impl<'id, T> AddAssign<&T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    fn add_assign(&mut self, rhs: &T) {
        *self = *self + *rhs;
    }
}

// by-value: Iterator<Item = L>
impl<'id, T> Sum for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::N, |acc, x| acc + x)
    }
}

// by-ref: Iterator<Item = &L>
impl<'id, 'a, T> Sum<&'a V<'id, T>> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn sum<I: Iterator<Item = &'a V<'id, T>>>(iter: I) -> Self {
        iter.fold(Self::N, |acc, x| acc + *x)
    }
}

impl<'id, T> Product for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::N, |acc, x| acc * x)
    }
}

impl<'id, 'a, T> Product<&'a V<'id, T>> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn product<I: Iterator<Item = &'a V<'id, T>>>(iter: I) -> Self {
        iter.fold(Self::N, |acc, x| acc * *x)
    }
}
