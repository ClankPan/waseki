use num_traits::{One, Zero};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, hash_map::Entry},
    ops::{Add, Mul},
};

pub type M<T> = HashMap<usize, T>;
// pub type Exp<T> = (Option<(M<T>, M<T>)>, M<T>, Option<usize>);

#[derive(Debug, Clone)]
pub struct Arena<T> {
    pub(crate) auxes: RefCell<Vec<T>>,
    pub(crate) wires: RefCell<HashMap<usize, Exp<T>>>,
    pub(crate) exprs: RefCell<Vec<Exp<T>>>,
    pub(crate) input: RefCell<HashSet<usize>>,
    pub(crate) build: bool,
}

#[derive(Debug, Clone)]
pub enum Exp<T> {
    L(M<T>),
    Q(M<T>, M<T>, M<T>),
}

impl<T: One> Default for Arena<T> {
    fn default() -> Self {
        Self {
            auxes: RefCell::new(vec![T::one()]), // 定数の1
            wires: RefCell::new(HashMap::new()),
            exprs: RefCell::new(Vec::new()),
            input: RefCell::new(HashSet::from([0])), // 定数の1は常にinput
            build: true,
        }
    }
}

impl<T: Copy + One + Zero + PartialEq> Arena<T> {
    #[inline]
    pub fn disable(&self) {
        self.auxes.borrow_mut()[0] = T::zero(); // 定数項をゼロに
    }

    #[inline]
    pub fn enable(&self) {
        self.auxes.borrow_mut()[0] = T::one(); // 定数項を戻す
    }

    #[inline]
    pub fn alloc(&self, v: T) -> usize {
        let mut wit = self.auxes.borrow_mut();
        let idx = wit.len();
        wit.push(v);
        idx
    }

    #[inline]
    pub fn input(&self, idx: usize) {
        self.input.borrow_mut().insert(idx);
    }

    #[inline]
    pub fn wire(
        &self,
        q: Option<(Vec<(usize, T)>, Vec<(usize, T)>)>,
        l: Vec<(usize, T)>,
        idx: Option<usize>,
    ) {
        // build時以外は無用なコピーや保存を避ける
        if self.build {
            let exp = if let Some((a, b)) = q {
                let c = l;
                let (mut a, mut b, mut c) = (sum_by_key(a), sum_by_key(b), sum_by_key(c));
                if let Some(l) = linearize(&a, &b) {
                    merge_maps(&mut c, &l);
                    self.apply_subset(&mut c);
                    Exp::L(c)
                } else {
                    self.apply_subset(&mut a);
                    self.apply_subset(&mut b);
                    self.apply_subset(&mut c);
                    Exp::Q(a, b, c)
                }
            } else {
                let mut l = sum_by_key(l);
                self.apply_subset(&mut l);
                Exp::L(l)
            };

            if let Some(idx) = idx {
                self.wires.borrow_mut().insert(idx, exp);
            } else {
                self.exprs.borrow_mut().push(exp);
            }
        }
    }

    pub fn apply_subset(&self, m: &mut M<T>) {
        let mut s = HashMap::new();
        for k in m.keys().copied().collect::<Vec<_>>() {
            if let Some(Exp::L(l)) = self.wires.borrow().get(&k) {
                merge_maps(&mut s, l);
                m.remove(&k);
            }
        }
        merge_maps(m, &s);
    }

    #[inline]
    pub fn into_inner(self) -> (Vec<T>, HashMap<usize, Exp<T>>, Vec<Exp<T>>, HashSet<usize>) {
        let wit = self.auxes.into_inner();
        let alloc = self.wires.into_inner();
        let equal = self.exprs.into_inner();
        let input = self.input.into_inner();
        (wit, alloc, equal, input)
    }
}

fn sum_by_key<T>(a: Vec<(usize, T)>) -> HashMap<usize, T>
where
    T: Add<Output = T> + Copy + Zero + PartialEq,
{
    let mut map = HashMap::new();
    for (k, v) in a {
        map.entry(k).and_modify(|acc| *acc = *acc + v).or_insert(v);
    }
    clean_zero(&mut map);
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

/// 0項を削除
pub fn clean_zero<T>(m: &mut M<T>)
where
    T: Zero + PartialEq,
{
    m.retain(|_, v| !v.is_zero());
}
