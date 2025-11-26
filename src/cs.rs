use crate::{
    L, List,
    ar::Arena,
    r1cs::{R1CS, compile},
    var::Var as V,
};
use num_traits::{One, Zero};
use std::{iter::Sum, marker::PhantomData, ops::Neg};

#[derive(Default)]
pub struct ConstraintSystem<T> {
    pub r1cs: Option<R1CS<T>>,
    pub witness: Vec<T>,
}

impl<T> ConstraintSystem<T>
where
    T: Clone + Copy + Default + PartialEq + One + Zero + Neg<Output = T> + Sum + std::fmt::Debug,
{
    pub fn synthesize_with<R, F>(&mut self, f: F) -> R
    where
        F: for<'id> FnOnce(ConstraintSynthesizer<'id, T>) -> R,
        T: One + Zero + Copy + PartialEq + std::fmt::Debug,
    {
        self.witness.clear();
        let ar = Arena::<T>::default();
        let cs = ConstraintSynthesizer::new(&ar);
        let r = f(cs);

        let (auxes, wires, exprs, io) = ar.into_inner();

        self.witness = if let Some(r1cs) = &self.r1cs {
            r1cs.witness(auxes)
        } else {
            let r1cs = compile(&auxes, wires, exprs, io);
            let witness = r1cs.witness(auxes);
            self.r1cs = Some(r1cs);
            witness
        };

        return r;
    }

    pub fn is_satisfied(&self) -> bool {
        if let Some(r1cs) = &self.r1cs {
            r1cs.satisfies(&self.witness)
        } else {
            false
        }
    }
}

thread_local! {
    static LOCAL_STATE: RefCell<> = RefCell::new();
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
    pub fn new(ar: &'id Arena<T>) -> Self {
        Self {
            ar,
            _brand: PhantomData::<&mut ()>,
        }
    }

    #[inline]
    pub fn alloc<U>(&self, v: U) -> V<'id, T>
    where
        U: Into<T>,
    {
        V::L(L::alloc(self.ar, v.into()))
    }

    #[inline]
    pub fn input<U>(&self, v: U) -> V<'id, T>
    where
        U: Into<T>,
    {
        let v = v.into();
        let idx = self.ar.alloc(v);
        let ar = self.ar;
        let l = List::new((idx, T::one()));
        ar.input.borrow_mut().insert(idx);
        V::L(L { v, l, ar })
    }

    #[inline]
    pub fn constant<U>(&self, t: U) -> V<'id, T>
    where
        U: Into<T>,
    {
        V::L(L::constant(self.ar, t.into()))
    }

    #[inline]
    pub fn equal(&self, x: V<'id, T>, y: V<'id, T>) {
        x.equals(y);
    }

    #[inline]
    pub fn inputize(&self, v: V<'id, T>) {
        v.inputize();
    }

    #[inline]
    pub fn outputize(&self, v: V<'id, T>) {
        v.outputize();
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
    pub fn disabled(&self, b: bool) {
        self.ar.disabled(b);
    }

    pub fn synthesize_with<Snd, Ret, Fun>(&self, f: Fun) -> SynthesizerOutput<'id, T, Ret>
    where
        Fun: for<'s> FnOnce(ConstraintSynthesizer<'s, Snd>) -> Ret,
        Snd: One + Zero + Copy + PartialEq + std::fmt::Debug + Into<T>,
    {
        let ar = Arena::<Snd>::default();
        let cs = ConstraintSynthesizer {
            ar: &ar,
            _brand: PhantomData::<&mut ()>,
        };
        let ret = f(cs);

        // let base = self.ar.auxes.len();
        // self.ar.wires += ret.ar.wires.iter().map(|k,v| k += base, v.map(|v| v+=base) )
        // self.ar.exprs += ret.ar.exprs.iter().map(|v| v.map(|v| v+=base) )
        // self.ar.auxes += ret.auxes.map(into)
        //
        //

        todo!()
    }
}

pub struct SynthesizerOutput<'id, T, Ret> {
    pub one: V<'id, T>,
    pub input_vars: Vec<V<'id, T>>,
    pub output_vars: Vec<V<'id, T>>,
    pub ret: Ret,
}
