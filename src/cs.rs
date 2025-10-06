use crate::{
    L, List,
    ar::{Arena, M},
    r1cs::{R1CS, compile, optimize},
    var::V,
};
use num_traits::{One, Zero};
use std::{marker::PhantomData, ops::Neg};

#[derive(Default)]
pub struct ConstraintSystem<T> {
    r1cs: Option<R1CS<T>>,
}

impl<T> ConstraintSystem<T>
where
    T: Clone + Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    pub fn with_cs<R, F>(&mut self, f: F) -> R
    where
        F: for<'id> FnOnce(ConstraintSynthesizer<'id, T>) -> R,
        T: One + Zero + Copy + PartialEq + std::fmt::Debug,
    {
        let arena = Arena::<T>::default();
        let cs = ConstraintSynthesizer {
            ar: &arena,
            _brand: PhantomData::<&mut ()>,
        };
        let r = f(cs);

        compile(arena);

        return r;
    }
}

#[derive(Copy, Clone)]
pub struct ConstraintSynthesizer<'id, T> {
    pub ar: &'id Arena<T>,
    _brand: PhantomData<&'id mut ()>, // 不変ブランド
}

impl<'id, T> ConstraintSynthesizer<'id, T>
where
    T: Clone + Copy + Default + PartialEq + One + Zero + Neg<Output = T>,
{
    #[inline]
    pub fn alloc(&self, v: T) -> V<'id, T> {
        V::L(L::alloc(self.ar, v))
    }

    #[inline]
    pub fn input(&self, v: T) -> V<'id, T> {
        let idx = self.ar.alloc(v);
        let ar = self.ar;
        let l = List::new((idx, T::one()));
        ar.input.borrow_mut().insert(idx);
        V::L(L { v, l, ar })
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
    fn _inputize(&self, v: V<'id, T>) {
        v.inputize();
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
