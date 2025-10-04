use num_traits::{One, Zero};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Arena<T> {
    pub(crate) wit: RefCell<Vec<T>>,
    pub(crate) exp: RefCell<Vec<(Vec<(usize, T)>, Vec<(usize, T)>, Vec<(usize, T)>, usize)>>,
}

impl<T: One> Default for Arena<T> {
    fn default() -> Self {
        Self {
            wit: RefCell::new(vec![T::one()]), // 定数の1
            exp: RefCell::new(Vec::new()),
        }
    }
}

impl<T: One + Zero> Arena<T> {
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
    pub fn exp(&self, a: Vec<(usize, T)>, b: Vec<(usize, T)>, c: Vec<(usize, T)>, idx: usize) {
        self.exp.borrow_mut().push((a, b, c, idx));
    }
}
