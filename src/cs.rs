use crate::{L, N, ar::Arena};
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
    pub fn alloc(&self, v: T) -> L<'id, T> {
        let idx = self.ar.alloc(v);
        let mut l = [(0, T::zero()); N];
        l[0] = (idx, T::one());
        L { v, l, ar: self.ar }
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
