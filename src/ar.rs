use num_traits::{One, Zero};
use std::{cell::RefCell, collections::HashMap, ops::Add};

type M<T> = HashMap<usize, T>;
type Exp<T> = (M<T>, M<T>, M<T>, Option<usize>);

#[derive(Debug)]
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

impl<T: Copy + One + Zero> Arena<T> {
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
        let (a, b, c) = (sum_by_key(a), sum_by_key(b), sum_by_key(c));
        self.exp.borrow_mut().push((a, b, c, idx));
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
