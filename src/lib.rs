mod branched_list;

use std::cell::{Ref, RefCell};
use std::marker::PhantomData;
use std::ops::{Add, Mul};

/// ========== Arena（実データ保管所） ==========
struct Arena<T> {
    l_nodes: RefCell<Vec<Vec<T>>>,
    q_nodes: RefCell<Vec<QNode<T>>>,
    w: RefCell<Vec<T>>, // 例: R1CSテープ（お好みで構造化してOK）
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            l_nodes: RefCell::new(Vec::new()),
            q_nodes: RefCell::new(Vec::new()),
            w: RefCell::new(Vec::new()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QNode<T> {
    a: Vec<T>,
    b: Vec<T>,
    c: Vec<T>,
}

impl<T> Arena<T> {
    #[inline]
    fn push_l(&self, data: Vec<T>) -> u32 {
        let mut l = self.l_nodes.borrow_mut();
        let idx = l.len() as u32;
        l.push(data);
        idx
    }
    #[inline]
    fn push_q(&self, a: Vec<T>, b: Vec<T>, c: Vec<T>) -> u32 {
        let mut q = self.q_nodes.borrow_mut();
        let idx = q.len() as u32;
        q.push(QNode { a, b, c });
        idx
    }
    #[inline]
    fn borrow_l(&self, idx: u32) -> Vec<T>
    where
        T: Clone,
    {
        self.l_nodes.borrow()[idx as usize].clone()
    }
    #[inline]
    fn borrow_q(&self, idx: u32) -> QNode<T>
    where
        T: Clone,
    {
        self.q_nodes.borrow()[idx as usize].clone()
    }
    #[inline]
    fn append_w<I>(&self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.w.borrow_mut().extend(iter);
    }
    #[inline]
    fn view_w(&self) -> Ref<'_, Vec<T>> {
        self.w.borrow()
    }
}

/// ========== CS（ブランド付きハブ：アリーナへの参照を配布） ==========
pub fn with_cs<T, R, F>(f: F) -> R
where
    F: for<'id> FnOnce(CS<'id, T>) -> R,
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
    _brand: PhantomData<&'id mut ()>, // generative brand（不変）
}

impl<'id, T: Clone> CS<'id, T> {
    #[inline]
    pub fn make_l(&self, data: Vec<T>) -> L<'id, T> {
        L {
            idx: self.ar.push_l(data),
            ar: self.ar,
        }
    }
    #[inline]
    pub fn make_q(&self, a: Vec<T>, b: Vec<T>, c: Vec<T>) -> Q<'id, T> {
        Q {
            idx: self.ar.push_q(a, b, c),
            ar: self.ar,
        }
    }
    #[inline]
    pub fn view_w(&self) -> Ref<'_, Vec<T>> {
        self.ar.view_w()
    }
}

/// ========== ハンドル型（Copy） ==========
/// L/Q は (index, arena参照) だけ持つ → Copy 可能
#[derive(Copy, Clone)]
pub struct L<'id, T: Clone> {
    idx: u32,
    ar: &'id Arena<T>,
}

#[derive(Copy, Clone)]
pub struct Q<'id, T: Clone> {
    idx: u32,
    ar: &'id Arena<T>,
}

/// 便利メソッド（内部読み出し）
impl<'id, T: Clone> L<'id, T> {
    #[inline]
    fn load(&self) -> Vec<T> {
        self.ar.borrow_l(self.idx)
    }
}
impl<'id, T: Clone> Q<'id, T> {
    #[inline]
    fn load(&self) -> QNode<T> {
        self.ar.borrow_q(self.idx)
    }
}

/// ========== コア演算（ハンドル→アリーナ操作） ==========

#[inline]
fn new_l<'id, T: Clone>(ar: &'id Arena<T>, data: Vec<T>) -> L<'id, T> {
    L {
        idx: ar.push_l(data),
        ar,
    }
}

#[inline]
fn new_q<'id, T: Clone>(ar: &'id Arena<T>, a: Vec<T>, b: Vec<T>, c: Vec<T>) -> Q<'id, T> {
    Q {
        idx: ar.push_q(a, b, c),
        ar,
    }
}

#[inline]
fn l_add_l<'id, T: Clone>(x: L<'id, T>, y: L<'id, T>) -> L<'id, T> {
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    let mut v = x.load();
    v.extend(y.load());
    new_l(x.ar, v)
}

#[inline]
fn l_mul_l<'id, T: Clone>(x: L<'id, T>, y: L<'id, T>) -> Q<'id, T> {
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    new_q(x.ar, x.load(), y.load(), Vec::new())
}

#[inline]
fn q_add_l<'id, T: Clone>(q: Q<'id, T>, l: L<'id, T>) -> Q<'id, T> {
    debug_assert!(std::ptr::eq(q.ar as *const _, l.ar as *const _));
    let QNode { a, b, mut c } = q.load();
    c.extend(l.load());
    new_q(q.ar, a, b, c)
}

/// reduce: Q を R1CS 1行としてテープに出力し、L（一時）に簡約
#[inline]
pub fn reduce<'id, T: Clone>(q: Q<'id, T>) -> L<'id, T> {
    let QNode { a, b, c } = q.load();
    // 例: a|b|c をそのまま "テープ" に追記（実装では A*B=C を直列化）
    q.ar.append_w(a.clone());
    q.ar.append_w(b.clone());
    q.ar.append_w(c.clone());

    let mut data = a;
    data.extend(b);
    data.extend(c);
    new_l(q.ar, data)
}

#[inline]
fn q_mul_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> Q<'id, T> {
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    let lx = reduce(x);
    let ly = reduce(y);
    l_mul_l(lx, ly) // a=lx, b=ly, c=[]
}

#[inline]
fn q_add_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> L<'id, T> {
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    let lx = reduce(x);
    let ly = reduce(y);
    l_add_l(lx, ly)
}

/// ========== 演算子実装（ハンドルは Copy なので owned-owned だけでOK） ==========

impl<'id, T: Clone> Add for L<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        l_add_l(self, rhs)
    }
}
impl<'id, T: Clone> Mul for L<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        l_mul_l(self, rhs)
    }
}
impl<'id, T: Clone> Add<L<'id, T>> for Q<'id, T> {
    type Output = Q<'id, T>;
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        q_add_l(self, rhs)
    }
}
// Q*Q -> Q（内部 reduce）
impl<'id, T: Clone> Mul for Q<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        q_mul_q(self, rhs)
    }
}
// Q+Q -> L（内部 reduce）
impl<'id, T: Clone> Add for Q<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        q_add_q(self, rhs)
    }
}

/// ========== 簡単なデモ ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo() {
        with_cs::<i32, _, _>(|cs| {
            let l1 = cs.make_l(vec![1, 2]);
            let l2 = cs.make_l(vec![3]);

            let l = l1 + l2; // L + L -> L
            let q = l * l1; // L * L -> Q
            let q = q + l; // Q + L -> Q
            let _q = q * (l1 * l2); // Q * Q -> Q（内部 reduce）

            assert!(!cs.view_w().is_empty()); // reduce によりテープに出力されている
        });
    }
}
