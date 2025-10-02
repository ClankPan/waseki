// src/lib.rs
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Add, Mul};

/// ------------------------------------------------------------
/// ブランド付き CS：呼ぶたびに新しい 'id を導入（generative）
/// ------------------------------------------------------------
pub fn with_cs<T, R, F>(f: F) -> R
where
    F: for<'id> FnOnce(CS<'id, T>) -> R,
{
    let w = RefCell::<Vec<T>>::new(Vec::new());
    let cs = CS {
        w: &w,
        _brand: PhantomData,
    };
    f(cs)
}

/// 制約テープ（ここでは Vec<T> をテープとして使う例）
#[derive(Debug)]
pub struct CS<'id, T> {
    w: &'id RefCell<Vec<T>>,
    // 'id を不変として保持（変位の抜け道を塞ぐ）
    _brand: PhantomData<&'id mut ()>,
}

impl<'id, T: Clone> CS<'id, T> {
    #[inline]
    pub fn make_l(&self, data: Vec<T>) -> L<'id, T> {
        L { data, cs: self.w }
    }
    #[inline]
    pub fn make_q(&self, a: Vec<T>, b: Vec<T>, c: Vec<T>) -> Q<'id, T> {
        Q {
            a,
            b,
            c,
            cs: self.w,
        }
    }
    #[inline]
    pub fn view_w(&self) -> std::cell::Ref<'_, Vec<T>> {
        self.w.borrow()
    }
}

/// 線形（和で増える）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct L<'id, T: Clone> {
    pub data: Vec<T>,
    cs: &'id RefCell<Vec<T>>,
}

/// 積ノード（A*B=C の未確定制約）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Q<'id, T: Clone> {
    pub a: Vec<T>,
    pub b: Vec<T>,
    pub c: Vec<T>,
    cs: &'id RefCell<Vec<T>>,
}

/* ========= ユーティリティ ========= */

#[inline]
fn concat_clone<T: Clone>(xs: &[T], ys: &[T]) -> Vec<T> {
    let mut v = Vec::with_capacity(xs.len() + ys.len());
    v.extend_from_slice(xs);
    v.extend_from_slice(ys);
    v
}

/// Q を R1CS 1 行としてテープに出力し、L に簡約（reduction）
#[inline]
pub fn reduce<'id, T: Clone>(q: &Q<'id, T>) -> L<'id, T> {
    {
        let mut w = q.cs.borrow_mut();
        // 実案件では a,b,c を R1CS 行としてシリアライズして push する
        w.extend(q.a.iter().cloned());
        w.extend(q.b.iter().cloned());
        w.extend(q.c.iter().cloned());
    }
    let mut data = Vec::with_capacity(q.a.len() + q.b.len() + q.c.len());
    data.extend_from_slice(&q.a);
    data.extend_from_slice(&q.b);
    data.extend_from_slice(&q.c);
    L { data, cs: q.cs }
}

/* ========= コア計算（参照版に集約） ========= */

#[inline]
fn l_add_l<'id, T: Clone>(x: &L<'id, T>, y: &L<'id, T>) -> L<'id, T> {
    L {
        data: concat_clone(&x.data, &y.data),
        cs: x.cs,
    }
}

#[inline]
fn l_mul_l<'id, T: Clone>(x: &L<'id, T>, y: &L<'id, T>) -> Q<'id, T> {
    Q {
        a: x.data.clone(),
        b: y.data.clone(),
        c: Vec::new(),
        cs: x.cs,
    }
}

#[inline]
fn q_add_l<'id, T: Clone>(q: &Q<'id, T>, l: &L<'id, T>) -> Q<'id, T> {
    let mut c = Vec::with_capacity(q.c.len() + l.data.len());
    c.extend_from_slice(&q.c);
    c.extend_from_slice(&l.data);
    Q {
        a: q.a.clone(),
        b: q.b.clone(),
        c,
        cs: q.cs,
    }
}

#[inline]
fn q_mul_q<'id, T: Clone>(q1: &Q<'id, T>, q2: &Q<'id, T>) -> Q<'id, T> {
    let l1 = reduce(q1);
    let l2 = reduce(q2);
    Q {
        a: l1.data,
        b: l2.data,
        c: Vec::new(),
        cs: q1.cs,
    }
}

#[inline]
fn q_add_q<'id, T: Clone>(q1: &Q<'id, T>, q2: &Q<'id, T>) -> L<'id, T> {
    let l1 = reduce(q1);
    let l2 = reduce(q2);
    L {
        data: concat_clone(&l1.data, &l2.data),
        cs: q1.cs,
    }
}

/* ========= 演算子トレイト実装（4パターン forward） ========= */
/* -- L + L -> L -- */
impl<'id, T: Clone> Add<&L<'id, T>> for &L<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: &L<'id, T>) -> Self::Output {
        l_add_l(self, rhs)
    }
}
impl<'id, T: Clone> Add<L<'id, T>> for &L<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        l_add_l(self, &rhs)
    }
}
impl<'id, T: Clone> Add<&L<'id, T>> for L<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: &L<'id, T>) -> Self::Output {
        l_add_l(&self, rhs)
    }
}
impl<'id, T: Clone> Add<L<'id, T>> for L<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        l_add_l(&self, &rhs)
    }
}

/* -- L * L -> Q -- */
impl<'id, T: Clone> Mul<&L<'id, T>> for &L<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: &L<'id, T>) -> Self::Output {
        l_mul_l(self, rhs)
    }
}
impl<'id, T: Clone> Mul<L<'id, T>> for &L<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        l_mul_l(self, &rhs)
    }
}
impl<'id, T: Clone> Mul<&L<'id, T>> for L<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: &L<'id, T>) -> Self::Output {
        l_mul_l(&self, rhs)
    }
}
impl<'id, T: Clone> Mul<L<'id, T>> for L<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        l_mul_l(&self, &rhs)
    }
}

/* -- Q + L -> Q -- */
impl<'id, T: Clone> Add<&L<'id, T>> for &Q<'id, T> {
    type Output = Q<'id, T>;
    fn add(self, rhs: &L<'id, T>) -> Self::Output {
        q_add_l(self, rhs)
    }
}
impl<'id, T: Clone> Add<L<'id, T>> for &Q<'id, T> {
    type Output = Q<'id, T>;
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        q_add_l(self, &rhs)
    }
}
impl<'id, T: Clone> Add<&L<'id, T>> for Q<'id, T> {
    type Output = Q<'id, T>;
    fn add(self, rhs: &L<'id, T>) -> Self::Output {
        q_add_l(&self, rhs)
    }
}
impl<'id, T: Clone> Add<L<'id, T>> for Q<'id, T> {
    type Output = Q<'id, T>;
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        q_add_l(&self, &rhs)
    }
}

/* -- Q * Q -> Q（内部で reduce→L*L） -- */
impl<'id, T: Clone> Mul<&Q<'id, T>> for &Q<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: &Q<'id, T>) -> Self::Output {
        q_mul_q(self, rhs)
    }
}
impl<'id, T: Clone> Mul<Q<'id, T>> for &Q<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: Q<'id, T>) -> Self::Output {
        q_mul_q(self, &rhs)
    }
}
impl<'id, T: Clone> Mul<&Q<'id, T>> for Q<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: &Q<'id, T>) -> Self::Output {
        q_mul_q(&self, rhs)
    }
}
impl<'id, T: Clone> Mul<Q<'id, T>> for Q<'id, T> {
    type Output = Q<'id, T>;
    fn mul(self, rhs: Q<'id, T>) -> Self::Output {
        q_mul_q(&self, &rhs)
    }
}

/* -- Q + Q -> L（内部で reduce→L + L） -- */
impl<'id, T: Clone> Add<&Q<'id, T>> for &Q<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: &Q<'id, T>) -> Self::Output {
        q_add_q(self, rhs)
    }
}
impl<'id, T: Clone> Add<Q<'id, T>> for &Q<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: Q<'id, T>) -> Self::Output {
        q_add_q(self, &rhs)
    }
}
impl<'id, T: Clone> Add<&Q<'id, T>> for Q<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: &Q<'id, T>) -> Self::Output {
        q_add_q(&self, rhs)
    }
}
impl<'id, T: Clone> Add<Q<'id, T>> for Q<'id, T> {
    type Output = L<'id, T>;
    fn add(self, rhs: Q<'id, T>) -> Self::Output {
        q_add_q(&self, &rhs)
    }
}

/* ========= 簡単なデモ ========= */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_flow() {
        with_cs::<i32, _, _>(|cs| {
            let l1 = cs.make_l(vec![1, 2]);
            let l2 = cs.make_l(vec![3]);

            let l_sum = l1.clone() + l2.clone(); // L + L -> L
            let q = l_sum.clone() * l2.clone(); // L * L -> Q
            let q2 = q.clone() + l1.clone(); // Q + L -> Q
            let _q3 = q2.clone() * (l1 * l2); // Q * Q -> Q（内部 reduce）

            let w = cs.view_w();
            assert!(w.len() > 0);
        });
    }
}
