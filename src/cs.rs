use crate::{L, List, N, ar::Arena, var::V};
use num_traits::{One, Zero};
use std::{marker::PhantomData, ops::Neg};

/// ========== CS（ブランド付き：generative lifetime） ==========
pub fn with_cs<T, R, F>(f: F) -> R
where
    F: for<'id> FnOnce(CS<'id, T>) -> R,
    T: One,
{
    let arena = Arena::<T>::default();
    let cs = CS {
        ar: &arena,
        _brand: PhantomData::<&mut ()>,
    };
    f(cs)
}

#[derive(Copy, Clone)]
pub struct CS<'id, T> {
    pub ar: &'id Arena<T>,
    _brand: PhantomData<&'id mut ()>, // 不変ブランド
}

impl<'id, T> CS<'id, T>
where
    T: Clone + Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    #[inline]
    pub fn alloc(&self, v: T) -> V<'id, T> {
        V::L(L::alloc(self.ar, v))
    }

    #[inline]
    pub fn constant(&self, t: T) -> V<'id, T> {
        V::L(L::constant(self.ar, t))
    }

    #[inline]
    pub fn equal(&self, x: V<'id, T>, y: V<'id, T>) {
        let v = x - y;
        let (a, b, c, idx) = match v {
            V::N => return,
            V::L(l) => (vec![], vec![], l.l.to_vec(), None),
            V::Q(q) => (q.a.l.to_vec(), q.b.l.to_vec(), q.c.l.to_vec(), None),
        };
        self.ar.wire(a, b, c, idx);
    }

    #[inline]
    pub fn one(&self) -> V<'id, T> {
        self.constant(T::one())
    }
    #[inline]
    pub fn zero(&self) -> V<'id, T> {
        self.constant(T::zero())
    }

    #[inline]
    pub fn disable(&self) {
        self.ar.disable();
    }

    #[inline]
    pub fn enable(&self) {
        self.ar.enable();
    }

    #[inline]
    pub fn export(self) -> Arena<T> {
        self.ar
    }
}
