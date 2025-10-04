use num_traits::{One, Zero};
use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    ops::{Add, Mul},
};

pub type M<T> = HashMap<usize, T>;
pub type Exp<T> = (Option<(M<T>, M<T>)>, M<T>, Option<usize>);

#[derive(Debug, Clone)]
pub struct Arena<T> {
    pub(crate) wit: RefCell<Vec<T>>,
    pub(crate) exp: RefCell<Vec<Exp<T>>>,
}

impl<T: One> Default for Arena<T> {
    fn default() -> Self {
        Self {
            wit: RefCell::new(vec![T::one()]), // 定数の1
            exp: RefCell::new(Vec::new()),
        }
    }
}

impl<T: Copy + One + Zero + PartialEq> Arena<T> {
    #[inline]
    pub fn disable(&self) {
        self.wit.borrow_mut()[0] = T::zero(); // 定数項をゼロに
    }

    #[inline]
    pub fn enable(&self) {
        self.wit.borrow_mut()[0] = T::one(); // 定数項を戻す
    }

    #[inline]
    pub fn alloc(&self, v: T) -> usize {
        let mut wit = self.wit.borrow_mut();
        let idx = wit.len();
        wit.push(v);
        idx
    }

    #[inline]
    pub fn wire(
        &self,
        a: Vec<(usize, T)>,
        b: Vec<(usize, T)>,
        c: Vec<(usize, T)>,
        idx: Option<usize>,
    ) {
        let (a, b, mut c) = (sum_by_key(a), sum_by_key(b), sum_by_key(c));
        if let Some(l) = linearize(&a, &b) {
            merge_maps(&mut c, &l);
            self.exp.borrow_mut().push((None, c, idx));
        } else {
            self.exp.borrow_mut().push((Some((a, b)), c, idx));
        }
    }

    #[inline]
    pub fn into_inner(self) -> (Vec<T>, Vec<Exp<T>>) {
        let wit = self.wit.into_inner();
        let exp = self.exp.into_inner();
        (wit, exp)
    }
}

fn sum_by_key<T>(a: Vec<(usize, T)>) -> HashMap<usize, T>
where
    T: Add<Output = T> + Copy,
{
    let mut map = HashMap::new();
    for (k, v) in a {
        map.entry(k).and_modify(|acc| *acc = *acc + v).or_insert(v);
    }
    map
}

/// A または B のどちらかが {0:c} だけなら、もう片方を c 倍して返す
fn linearize<T>(a: &M<T>, b: &M<T>) -> Option<M<T>>
where
    T: Copy + Zero + Mul<Output = T>,
{
    let only_const = |m: &M<T>| (m.len() == 1).then(|| m.get(&0).copied()).flatten();

    match (only_const(a), only_const(b)) {
        (Some(c), None) => Some(scale_map(b, c)),
        (None, Some(c)) => Some(scale_map(a, c)),
        _ => None,
    }
}

/// m を c 倍した新しい M を返す
fn scale_map<T>(m: &M<T>, c: T) -> M<T>
where
    T: Copy + Mul<Output = T>,
{
    let mut out = M::with_capacity(m.len());
    for (&k, &v) in m {
        out.insert(k, v * c);
    }
    out
}

/// b を a に加算（0 は保持しない）
pub fn merge_maps<T>(a: &mut M<T>, b: &M<T>)
where
    T: Copy + Zero + PartialEq + Add<Output = T>,
{
    for (&k, &vb) in b {
        match a.entry(k) {
            Entry::Vacant(v) => {
                if vb != T::zero() {
                    v.insert(vb);
                }
            }
            Entry::Occupied(mut o) => {
                let cur = *o.get();
                let nxt = cur + vb;
                if nxt == T::zero() {
                    o.remove();
                } else {
                    *o.get_mut() = nxt;
                }
            }
        }
    }
}
