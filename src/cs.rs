use crate::{L, ar::Arena, r1cs::optimize, var::V};
use num_traits::{One, Zero};
use std::{marker::PhantomData, ops::Neg};

/// ========== CS（ブランド付き：generative lifetime） ==========
pub fn with_cs<T, R, F>(f: F) -> R
where
    F: for<'id> FnOnce(CS<'id, T>) -> R,
    T: One + Zero + Copy + PartialEq + std::fmt::Debug,
{
    let arena = Arena::<T>::default();
    let cs = CS {
        ar: &arena,
        _brand: PhantomData::<&mut ()>,
    };
    let r = f(cs);

    optimize(arena);

    return r;
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
        match v {
            V::N => return,
            V::L(l) => self.ar.wire(None, l.l.to_vec(), None),
            V::Q(q) => self
                .ar
                .wire(Some((q.a.l.to_vec(), q.b.l.to_vec())), q.c.l.to_vec(), None),
        };
    }

    #[inline]
    fn inputize(&self, v: V<'id, T>) {
        let idx = match v {
            V::N => return,
            V::L(l) => {
                let idx = self.ar.alloc(l.v);
                self.ar.wire(None, l.l.to_vec(), Some(idx));
                idx
            }
            V::Q(q) => {
                let (a, b, c) = (q.a, q.b, q.c);
                let v = a.v * b.v + c.v;
                let idx = self.ar.alloc(v);
                self.ar
                    .wire(Some((a.l.to_vec(), b.l.to_vec())), c.l.to_vec(), Some(idx));
                idx
            }
        };
        self.ar.input.borrow_mut().insert(idx);
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
