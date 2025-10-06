mod ar;
mod cs;
mod debug;
mod ops;
mod r1cs;
#[cfg(test)]
mod tests;
mod var;

use ar::Arena;
pub use cs::*;

use num_traits::{One, Zero};
use std::ops::{Add, Mul};

const N: usize = 3;

#[derive(Copy, Clone, Debug)]
struct List<T> {
    list: [(usize, T); N],
    len: usize,
}

impl<T: Copy + Zero> Default for List<T> {
    fn default() -> Self {
        Self {
            list: [(0, T::zero()); N],
            len: 0,
        }
    }
}

impl<T: Copy + One + Zero> List<T> {
    fn new(v: (usize, T)) -> Self {
        let mut l = Self::default();
        l.push(v);
        l
    }
    fn push(&mut self, v: (usize, T)) {
        self.list[self.len] = v;
        self.len += 1;
    }
    fn to_vec(&self) -> Vec<(usize, T)> {
        self.list[..self.len].to_vec()
    }
    fn mul(&mut self, t: T, build: bool) {
        if build {
            self.list.iter_mut().for_each(|i| i.1 = t * i.1);
        }
    }
    fn len(&self) -> usize {
        self.len
    }
    fn merge(&mut self, rhs: Self, build: bool) {
        if build {
            for v in rhs.to_vec() {
                self.push(v)
            }
        } else {
            // buildが無効の時は無用なコピーを避けるために長さだけ保持する
            self.len += rhs.len;
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct L<'id, T> {
    v: T,
    l: List<T>,
    ar: &'id Arena<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct Q<'id, T> {
    a: L<'id, T>,
    b: L<'id, T>,
    c: L<'id, T>,
    ar: &'id Arena<T>,
}

impl<'id, T: One + Zero + Copy + PartialEq> L<'id, T> {
    #[inline]
    fn new(ar: &'id Arena<T>) -> Self {
        Self {
            v: T::zero(),
            l: List::default(),
            ar,
        }
    }

    #[inline]
    fn alloc(ar: &'id Arena<T>, v: T) -> Self {
        let idx = ar.alloc(v);
        let l = List::new((idx, T::one()));
        L { v, l, ar }
    }

    #[inline]
    fn constant(ar: &'id Arena<T>, t: T) -> Self {
        let mut l = Self::new(ar);
        l.l = List::new((0, t));
        l.v = t;
        l
    }

    #[inline]
    pub fn value(&self) -> T {
        self.v
    }
}

impl<'id, T> Q<'id, T>
where
    T: Copy + Add<Output = T> + Mul<Output = T> + One + Zero + PartialEq,
{
    #[inline]
    pub fn reduce(&self) -> L<'id, T> {
        let (a, b, c) = (self.a, self.b, self.c);
        let v = a.v * b.v + c.v; // A*B+C=W
        let idx = self.ar.alloc(v);
        self.ar
            .wire(Some((a.l.to_vec(), b.l.to_vec())), c.l.to_vec(), Some(idx));
        let l = List::new((idx, T::one()));
        L { l, ar: self.ar, v }
    }

    #[inline]
    pub fn value(&self) -> T {
        self.a.v * self.b.v + self.c.v
    }
}

/// ========== L + L -> L ==========
#[inline]
fn l_add_l<'id, T>(mut x: L<'id, T>, y: L<'id, T>) -> L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));

    let v = x.v + y.v;
    let ar = x.ar;

    let l = if x.l.len() + y.l.len() > N {
        let idx = ar.alloc(v);
        ar.wire(None, [x.l.to_vec(), y.l.to_vec()].concat(), Some(idx));
        List::new((idx, T::one()))
    } else {
        x.l.merge(y.l, ar.build);
        x.l
    };

    L { l, ar: x.ar, v }
}

/// ========== L * L -> Q ==========
#[inline]
fn l_mul_l<'id, T>(a: L<'id, T>, b: L<'id, T>) -> Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(a.ar as *const _, b.ar as *const _));
    let ar = a.ar;
    let c = L::new(ar);
    Q { a, b, c, ar }
}

/// ========== Q + L -> Q ==========
#[inline]
fn q_add_l<'id, T: Clone>(q: Q<'id, T>, l: L<'id, T>) -> Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(q.ar as *const _, l.ar as *const _));
    let (a, b, c) = (q.a, q.b, l_add_l(q.c, l));
    let ar = q.ar;
    Q { a, b, c, ar }
}

/// ========== Q * L -> Q ==========
#[inline]
fn q_mul_l<'id, T: Clone>(q: Q<'id, T>, l: L<'id, T>) -> Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(q.ar as *const _, l.ar as *const _));
    l_mul_l(q.reduce(), l)
}

/// ========== Q + Q -> Q ==========
#[inline]
fn q_add_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    q_add_l(x, y.reduce())
}

/// ========== Q * Q -> Q ==========
#[inline]
fn q_mul_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    l_mul_l(x.reduce(), y.reduce())
}

/// ========== T * L -> L ==========
#[inline]
fn t_mul_l<'id, T: Clone>(t: T, l: L<'id, T>) -> L<'id, T>
where
    T: Copy + Default + One + Zero,
{
    let v = t * l.v;
    let ar = l.ar;
    let mut l = l.l;
    l.mul(t, ar.build);
    L { l, v, ar }
}

/// ========== T * Q -> Q ==========
#[inline]
fn t_mul_q<'id, T: Clone>(t: T, q: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Default + One + Zero,
{
    let a = q.a;
    let b = t_mul_l(t, q.b);
    let c = t_mul_l(t, q.c);
    let ar = q.ar;
    Q { a, b, c, ar }
}

/// ========== T + L -> L ==========
#[inline]
fn t_add_l<'id, T: Clone>(t: T, l: L<'id, T>) -> L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    let t = L::constant(l.ar, t);
    l_add_l(l, t)
}

/// ========== T + Q -> Q==========
#[inline]
fn t_add_q<'id, T: Clone>(t: T, q: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    let t = L::constant(q.ar, t);
    q_add_l(q, t)
}
