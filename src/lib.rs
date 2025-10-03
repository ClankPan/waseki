mod aops;
mod ar;
mod bops;
mod cs;
mod rops;
mod utils;

pub use aops::*;
use ar::Arena;
pub use bops::*;
pub use cs::*;
pub use rops::*;
use utils::*;

use num_traits::{One, Zero};
use std::ops::{Add, Mul};

const N: usize = 10;

#[derive(Copy, Clone)]
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

impl<T: Copy + Zero> List<T> {
    fn new(v: (usize, T)) -> Self {
        let mut l = Self::default();
        l.push(v);
        l
    }
    fn push(&mut self, v: (usize, T)) {
        self.list[self.len] = v;
        self.len += 1;
    }
}

#[derive(Copy, Clone)]
pub struct L<'id, T> {
    v: T,
    l: List<T>,
    ar: &'id Arena<T>,
}

#[derive(Copy, Clone)]
pub struct Q<'id, T> {
    a: L<'id, T>,
    b: L<'id, T>,
    c: L<'id, T>,
    ar: &'id Arena<T>,
}

impl<'id, T: Zero + Copy> L<'id, T> {
    #[inline]
    fn new(ar: &'id Arena<T>) -> Self {
        Self {
            v: T::zero(),
            l: List::default(),
            ar,
        }
    }
    #[inline]
    fn constant(ar: &'id Arena<T>, t: T) -> Self {
        let mut l = Self::new(ar);
        l.l.push((0, t));
        l
    }

    #[inline]
    pub fn value(&self) -> T {
        self.v
    }
}

impl<'id, T> Q<'id, T>
where
    T: Copy + Add<Output = T> + Mul<Output = T> + PartialEq + Default + One + Zero,
{
    #[inline]
    pub fn reduce(&self) -> L<'id, T> {
        let (a, b, c) = (self.a, self.b, self.c);
        let v = a.v * b.v + c.v; // A*B+C=W
        let idx = self.ar.reduce(a.l.into(), b.l.into(), c.l.into(), v);
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
fn l_add_l<'id, T>(x: L<'id, T>, y: L<'id, T>) -> L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));

    let v = x.v + y.v;

    // 疎和（同一 index を合算）
    let merged = merge_and_prune(&x.l, &y.l);

    // 収まるなら固定長にパック、収まらなければ reduce で witness 化
    let l = if let Ok(packed) = pack_list(&merged) {
        packed
    } else {
        let idx = x.ar.reduce(List::default(), List::default(), merged, v);
        List::new((idx, T::one()))
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

/// ========== Q + Q -> L ==========
#[inline]
fn q_add_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    l_add_l(x.reduce(), y.reduce())
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
    for i in &mut l {
        i.1 = t * i.1;
    }
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

/* ========= Test ========= */
#[cfg(test)]
mod tests {
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

            // u128 + L -> L;
            let l = 1u128 + l;

            // u128 + L -> L;
            let _l = 1u128 + l;

            // u128 + Q -> Q;
            let q = 1u128 + q;

            // u128 + Q -> Q;
            let _q = 1u128 + q;

            let l = &l + &l;

            // reduce がテープに値を出力しているはず
            // assert!(!cs.view_w().is_empty());
        });
    }

    #[test]
    fn demo_with_goldilocks_ring_ntt() {
        demo::<GoldilocksRingNTT>()
    }
}

// if x.l++y.lの非ゼロの個数が2Nを超えたらreduceしてwitnessを割り当ててしまう
// その場合、二回目以降のloopのwitnessの割り当て順をどうするか？
// 最適化時にシャッフルしないでshrinkだけすれば一致するはず
