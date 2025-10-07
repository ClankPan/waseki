use num_traits::{One, Zero};
use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{
    L, Q, l_add_l, l_mul_l, q_add_l, q_add_q, q_mul_l, q_mul_q, t_add_l, t_add_q, t_mul_l, t_mul_q,
};

#[derive(Copy, Clone, Debug)]
pub enum V<'id, T> {
    N,
    L(L<'id, T>),
    Q(Q<'id, T>),
}

impl<'id, T> V<'id, T>
where
    T: One + Zero + Copy + PartialEq + Neg<Output = T> + Default,
{
    /// Creates an empty variable.
    pub fn new() -> Self {
        Self::N
    }

    /// Extracts inner value.
    pub fn value(&self) -> T {
        match self {
            V::L(l) => l.value(),
            V::Q(q) => q.value(),
            V::N => T::zero(),
        }
    }

    /// Marks this variable as **public input** of the circuit.
    pub fn inputize(&self) {
        match self {
            V::N => return,
            V::L(l) => {
                let idx = l.ar.alloc(l.v);
                l.ar.wire(None, l.l.to_vec(), Some(idx));
                l.ar.input(idx);
            }
            V::Q(q) => {
                let (a, b, c) = (q.a, q.b, q.c);
                let v = a.v * b.v + c.v;
                let idx = q.ar.alloc(v);
                q.ar.wire(Some((a.l.to_vec(), b.l.to_vec())), c.l.to_vec(), Some(idx));
                q.ar.input(idx);
            }
        };
    }

    /// Enforce this variable equals zero within the circuit.
    pub fn equals_zero(&self) {
        match self {
            V::N => return,
            V::L(l) => l.ar.wire(None, l.l.to_vec(), None),
            V::Q(q) => {
                q.ar.wire(Some((q.a.l.to_vec(), q.b.l.to_vec())), q.c.l.to_vec(), None)
            }
        };
    }

    /// Enforces equality between two variables in the circuit.
    pub fn equals(&self, rhs: Self) {
        (self - &rhs).equals_zero();
    }

    /// Enforce this variable equals the constant within the circuit.
    pub fn equals_const<U>(&self, t: U)
    where
        U: Into<T>,
    {
        (self - &t.into()).equals_zero();
    }
}

impl<'id, T> Neg for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    type Output = V<'id, T>;

    fn neg(self) -> Self::Output {
        let minus = -T::one();
        match self {
            V::N => V::N,
            V::L(l) => V::L(t_mul_l(minus, l)),
            V::Q(q) => V::Q(t_mul_q(minus, q)),
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
            (V::Q(x), V::Q(y)) => V::Q(q_add_q(x, y)),
            (V::L(l), V::Q(q)) | (V::Q(q), V::L(l)) => V::Q(q_add_l(q, l)),
            (V::N, V::L(l)) | (V::L(l), V::N) => V::L(l),
            (V::N, V::Q(q)) | (V::Q(q), V::N) => V::Q(q),
            (V::N, V::N) => V::N,
        }
    }
}

// V - V -> V
impl<'id, T> Sub for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    type Output = V<'id, T>;
    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs.neg()
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
            (V::Q(x), V::Q(y)) => V::Q(q_mul_q(x, y)),
            (V::L(l), V::Q(q)) | (V::Q(q), V::L(l)) => V::Q(q_mul_l(q, l)),
            (V::N, V::L(l)) | (V::L(l), V::N) => V::L(l),
            (V::N, V::Q(q)) | (V::Q(q), V::N) => V::Q(q),
            (V::N, V::N) => V::N,
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

// &V - &V -> V
impl<'id, T> Sub for &V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    type Output = V<'id, T>;
    fn sub(self, rhs: Self) -> Self::Output {
        *self + rhs.neg()
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

// V - T -> V
impl<'id, T> Sub<T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    type Output = V<'id, T>;
    fn sub(self, rhs: T) -> Self::Output {
        self + rhs.neg()
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

// &V + &T -> V
impl<'id, T> Add<&T> for &V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn add(self, rhs: &T) -> Self::Output {
        *self + *rhs
    }
}

// &V - &T -> V
impl<'id, T> Sub<&T> for &V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    type Output = V<'id, T>;
    fn sub(self, rhs: &T) -> Self::Output {
        *self + rhs.neg()
    }
}

// &V * &T -> V
impl<'id, T> Mul<&T> for &V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = V<'id, T>;
    fn mul(self, rhs: &T) -> Self::Output {
        *self * *rhs
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

// V -= V
impl<'id, T> SubAssign for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = &*self - &rhs;
    }
}

// V -= &V
impl<'id, T> SubAssign<&V<'id, T>> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: &Self) {
        *self = &*self - rhs;
    }
}

// V *= V
impl<'id, T> MulAssign for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = &*self * &rhs;
    }
}

// V *= &V
impl<'id, T> MulAssign<&V<'id, T>> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    fn mul_assign(&mut self, rhs: &Self) {
        *self = &*self * rhs;
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

// V -= T
impl<'id, T> SubAssign<T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - rhs;
    }
}

// V -= &T
impl<'id, T> SubAssign<&T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: &T) {
        *self = *self - *rhs;
    }
}

// V *= T
impl<'id, T> MulAssign<T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs;
    }
}

// V *= &T
impl<'id, T> MulAssign<&T> for V<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    fn mul_assign(&mut self, rhs: &T) {
        *self = *self * *rhs;
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
