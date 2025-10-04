use crate::{L, List, N, ar::Arena, var::V};
use num_traits::{One, Zero};
use std::marker::PhantomData;

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
    ar: &'id Arena<T>,
    _brand: PhantomData<&'id mut ()>, // 不変ブランド
}

impl<'id, T> CS<'id, T>
where
    T: Clone + Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    pub fn alloc(&self, v: T) -> V<'id, T> {
        let idx = self.ar.alloc(v);
        let l = List::new((idx, T::one()));
        V::L(L { v, l, ar: self.ar })
    }

    #[inline]
    pub fn constant(&self, t: T) -> V<'id, T> {
        V::L(L::constant(self.ar, t))
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
}
