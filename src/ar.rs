use num_traits::{One, Zero};
use std::cell::RefCell;

use crate::List;

pub struct Arena<T> {
    wit: RefCell<Vec<T>>,
    exp: RefCell<Vec<(Vec<(usize, T)>, Vec<(usize, T)>, Vec<(usize, T)>, usize)>>,
}

impl<T: One> Default for Arena<T> {
    fn default() -> Self {
        Self {
            wit: RefCell::new(vec![T::one()]), // 定数の1
            exp: RefCell::new(Vec::new()),
        }
    }
}

impl<T: Default + PartialEq + One + Zero> Arena<T> {
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

    // #[inline]
    // pub fn reduce(&self, a: List<T>, b: List<T>, c: List<T>, v: T) -> usize {
    //     let zero = T::default();
    //
    //     let mut a: Vec<_> = a.list.into();
    //     let mut b: Vec<_> = b.list.into();
    //     let mut c: Vec<_> = c.list.into();
    //
    //     // その場で 0 要素を除去（追加の Vec を作らない）
    //     a.retain(|(_, x)| x != &zero);
    //     b.retain(|(_, x)| x != &zero);
    //     c.retain(|(_, x)| x != &zero);
    //
    //     let idx = self.alloc(v);
    //     self.exp.borrow_mut().push((a, b, c, idx));
    //     idx
    // }
}
