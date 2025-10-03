use num_traits::{One, Zero};
use std::cell::RefCell;

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
    pub fn reduce(
        &self,
        mut a: Vec<(usize, T)>,
        mut b: Vec<(usize, T)>,
        mut c: Vec<(usize, T)>,
        v: T,
    ) -> usize {
        let zero = T::default();

        // その場で 0 要素を除去（追加の Vec を作らない）
        a.retain(|(_, x)| x != &zero);
        b.retain(|(_, x)| x != &zero);
        c.retain(|(_, x)| x != &zero);

        let idx = self.alloc(v);
        self.exp.borrow_mut().push((a, b, c, idx));
        idx
    }
}
