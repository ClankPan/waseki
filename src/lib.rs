use num_traits::{One, Zero};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Add, Mul};

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

impl<T: Default + PartialEq> Arena<T> {
    #[inline]
    fn alloc(&self, v: T) -> usize {
        let mut wit = self.wit.borrow_mut();
        let idx = wit.len();
        wit.push(v);
        idx
    }

    #[inline]
    fn reduce(
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

/// ========== ハンドル（Copy） ==========
/// L/Q は “インデックス + Arena参照” だけを持つ
const N: usize = 10;

type List<T> = [(usize, T); N];

#[derive(Copy, Clone)]
pub struct L<'id, T> {
    v: T,
    l: List<T>,
    ar: &'id Arena<T>,
}

#[derive(Copy, Clone)]
pub struct Q<'id, T> {
    a: L<'id, T>,
    b: L<'id, T>,
    c: L<'id, T>,
    ar: &'id Arena<T>,
}

impl<'id, T: Zero + Copy> L<'id, T> {
    #[inline]
    fn new(ar: &'id Arena<T>) -> Self {
        Self {
            v: T::zero(),
            l: [(0, T::zero()); N],
            ar,
        }
    }
    #[inline]
    fn constant(ar: &'id Arena<T>, t: T) -> Self {
        let mut l = Self::new(ar);
        l.l[0] = (0, t);
        l
    }
}

impl<'id, T> Q<'id, T>
where
    T: Copy + Add<Output = T> + Mul<Output = T> + PartialEq + Default + One + Zero,
{
    #[inline]
    pub fn reduce(&self) -> L<'id, T> {
        let (a, b, c) = (self.a, self.b, self.c);
        let v = a.v * b.v + c.v; // A*B+C=W
        let idx = self.ar.reduce(a.l.into(), b.l.into(), c.l.into(), v);
        let mut l = [(0, T::zero()); N];
        l[0] = (idx, T::one());
        L { l, ar: self.ar, v }
    }
}

/// ========== CS（ブランド付き：generative lifetime） ==========
pub fn with_cs<T, R, F>(f: F) -> R
where
    F: for<'id> FnOnce(CS<'id, T>) -> R,
    T: One,
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
    _brand: PhantomData<&'id mut ()>, // 不変ブランド
}

impl<'id, T> CS<'id, T>
where
    T: Clone + Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    pub fn alloc(&self, v: T) -> L<'id, T> {
        let idx = self.ar.alloc(v);
        let mut l = [(0, T::zero()); N];
        l[0] = (idx, T::one());
        L { l, ar: self.ar, v }
    }
}

/// 2つの疎ベクトル（固定長）を結合して、同じ index を合算し、0 を除去
fn merge_and_prune<T>(a: &List<T>, b: &List<T>) -> Vec<(usize, T)>
where
    T: Copy + Add<Output = T> + PartialEq + Default + Zero,
{
    let mut map: HashMap<usize, T> = HashMap::new();
    let zero = T::zero();

    for &(i, c) in a.iter().chain(b.iter()) {
        if c == zero {
            continue;
        }
        map.entry(i)
            .and_modify(|acc| *acc = (*acc) + c)
            .or_insert(c);
    }

    // 0 になったものを取り除く
    map.into_iter().filter(|&(_i, ref c)| *c != zero).collect()
}

/// 可変長 Vec を固定長 List<T> にパック（余りは 0 で埋める）
/// len > N の場合は Err(vec) を返す（呼び出し側で reduce へ）
fn pack_list<T>(v: &[(usize, T)]) -> Result<List<T>, ()>
where
    T: Copy + Default + Zero,
{
    if v.len() > N {
        return Err(());
    }
    let mut out = [(0usize, T::zero()); N];
    for (dst, &(i, c)) in out.iter_mut().zip(v.iter()) {
        *dst = (i, c);
    }
    Ok(out)
}

/// ========== L + L -> L ==========
#[inline]
fn l_add_l<'id, T>(x: L<'id, T>, y: L<'id, T>) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));

    let v = x.v + y.v;

    // 疎和（同一 index を合算）
    let merged = merge_and_prune(&x.l, &y.l);

    // 収まるなら固定長にパック、収まらなければ reduce で witness 化
    let l = if let Ok(packed) = pack_list(&merged) {
        packed
    } else {
        let idx = x.ar.reduce(vec![], vec![], merged, v);
        let mut tmp = [(0, T::zero()); N];
        tmp[0] = (idx, T::one());
        tmp
    };

    L { l, ar: x.ar, v }
}

/// ========== L * L -> Q ==========
#[inline]
fn l_mul_l<'id, T>(a: L<'id, T>, b: L<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Mul<Output = T> + One + Zero,
{
    debug_assert!(std::ptr::eq(a.ar as *const _, b.ar as *const _));
    let ar = a.ar;
    let c = L::new(ar);
    Q { a, b, c, ar }
}

/// ========== Q + L -> Q ==========
#[inline]
fn q_add_l<'id, T: Clone>(q: Q<'id, T>, l: L<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    debug_assert!(std::ptr::eq(q.ar as *const _, l.ar as *const _));
    let (a, b, c) = (q.a, q.b, l_add_l(q.c, l));
    let ar = q.ar;
    Q { a, b, c, ar }
}

/// ========== L + Q -> Q ==========
#[inline]
fn l_add_q<'id, T: Clone>(l: L<'id, T>, q: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    q_add_l(q, l)
}

/// ========== Q * L -> Q ==========
#[inline]
fn q_mul_l<'id, T: Clone>(q: Q<'id, T>, l: L<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Mul<Output = T> + One + Zero,
{
    debug_assert!(std::ptr::eq(q.ar as *const _, l.ar as *const _));
    l_mul_l(q.reduce(), l)
}

/// ========== Q + Q -> L ==========
#[inline]
fn q_add_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    l_add_l(x.reduce(), y.reduce())
}

/// ========== Q * Q -> Q ==========
#[inline]
fn q_mul_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
    l_mul_l(x.reduce(), y.reduce())
}

/// ========== T * L -> L ==========
#[inline]
fn t_mul_l<'id, T: Clone>(t: T, l: L<'id, T>) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let v = t * l.v;
    let ar = l.ar;
    let mut l = l.l;
    for i in &mut l {
        i.1 = t * i.1;
    }
    L { l, v, ar }
}

/// ========== L * T -> L ==========
#[inline]
fn l_mul_t<'id, T: Clone>(l: L<'id, T>, t: T) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    t_mul_l(t, l)
}

/// ========== T * Q -> Q ==========
#[inline]
fn t_mul_q<'id, T: Clone>(t: T, q: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let a = q.a;
    let b = t_mul_l(t, q.b);
    let c = t_mul_l(t, q.c);
    let ar = q.ar;
    Q { a, b, c, ar }
}

/// ========== Q * T -> Q ==========
#[inline]
fn q_mul_t<'id, T: Clone>(q: Q<'id, T>, t: T) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    t_mul_q(t, q)
}

/// ========== u128 * L -> L ==========
#[inline]
fn u128_mul_l<'id, T: Clone + From<u128>>(t: u128, l: L<'id, T>) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let t = T::from(t);
    t_mul_l(t, l)
}

/// ========== L * u128 -> L ==========
#[inline]
fn l_mul_u128<'id, T: Clone + From<u128>>(l: L<'id, T>, t: u128) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    u128_mul_l(t, l)
}

/// ========== u128 * Q -> Q ==========
#[inline]
fn u128_mul_q<'id, T: Clone + From<u128>>(t: u128, q: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let t = T::from(t);
    t_mul_q(t, q)
}

/// ========== Q * u128 -> Q ==========
#[inline]
fn q_mul_u128<'id, T: Clone + From<u128>>(q: Q<'id, T>, t: u128) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    u128_mul_q(t, q)
}

/// ========== T + L -> L ==========
#[inline]
fn t_add_l<'id, T: Clone>(t: T, l: L<'id, T>) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let t = L::constant(l.ar, t);
    l_add_l(t, l)
}

/// ========== L + T -> L ==========
#[inline]
fn l_add_t<'id, T: Clone>(l: L<'id, T>, t: T) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    t_add_l(t, l)
}

/// ========== T + Q -> Q==========
#[inline]
fn t_add_q<'id, T: Clone>(t: T, q: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let t = L::constant(q.ar, t);
    l_add_q(t, q)
}

/// ========== Q + T -> Q ==========
#[inline]
fn q_add_t<'id, T: Clone>(q: Q<'id, T>, t: T) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    t_add_q(t, q)
}

/// ========== u128 + L -> L ==========
#[inline]
fn u128_add_l<'id, T: Clone + From<u128>>(t: u128, l: L<'id, T>) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let t = T::from(t);
    t_add_l(t, l)
}

/// ========== L + u128 -> L ==========
#[inline]
fn l_add_u128<'id, T: Clone + From<u128>>(l: L<'id, T>, t: u128) -> L<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    u128_add_l(t, l)
}

/// ========== u128 + Q -> Q ==========
#[inline]
fn u128_add_q<'id, T: Clone + From<u128>>(t: u128, q: Q<'id, T>) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    let t = T::from(t);
    t_add_q(t, q)
}

/// ========== Q + u128 -> Q ==========
#[inline]
fn q_add_u128<'id, T: Clone + From<u128>>(q: Q<'id, T>, t: u128) -> Q<'id, T>
where
    T: Copy + Clone + Default + PartialEq + Add<Output = T> + One + Zero,
{
    u128_add_q(t, q)
}

/* ========= 演算子トレイト（Copyなので owned-owned でOK） ========= */
// L + L -> L
impl<'id, T> Add for L<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = L<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        l_add_l(self, rhs)
    }
}

// L * L -> Q
impl<'id, T> Mul for L<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = Q<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        l_mul_l(self, rhs)
    }
}

// L + Q -> Q
impl<'id, T> Add<Q<'id, T>> for L<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: Q<'id, T>) -> Self::Output {
        q_add_l(rhs, self)
    }
}

// L * Q -> Q
impl<'id, T> Mul<Q<'id, T>> for L<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: Q<'id, T>) -> Self::Output {
        // Q * L の実装をそのまま利用
        q_mul_l(rhs, self)
    }
}

// Q + L -> Q
impl<'id, T> Add<L<'id, T>> for Q<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = Q<'id, T>;
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        q_add_l(self, rhs)
    }
}

// Q * L -> Q
impl<'id, T> Mul<L<'id, T>> for Q<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = Q<'id, T>;
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        q_mul_l(self, rhs)
    }
}

// Q * Q -> Q
impl<'id, T> Mul for Q<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = Q<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        q_mul_q(self, rhs)
    }
}

// Q + Q -> Q
impl<'id, T: Clone> Add for Q<'id, T>
where
    T: Default + Clone + Copy + One + Zero + PartialEq,
{
    type Output = L<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        q_add_q(self, rhs)
    }
}

// u128 * L -> L
impl<'id, T> Mul<L<'id, T>> for u128
where
    T: Copy + Mul<Output = T> + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        u128_mul_l(self, rhs)
    }
}

// L * u128 -> L
impl<'id, T> Mul<u128> for L<'id, T>
where
    T: Copy + Mul<Output = T> + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: u128) -> Self::Output {
        u128_mul_l(rhs, self)
    }
}

pub struct C<T>(T); // to avoid orphan rules

// C * L -> L
impl<'id, T> Mul<L<'id, T>> for C<T>
where
    T: Copy + Mul<Output = T> + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        t_mul_l(self.0, rhs)
    }
}

// L * C -> L
impl<'id, T> Mul<C<T>> for L<'id, T>
where
    T: Copy + Mul<Output = T> + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: C<T>) -> Self::Output {
        t_mul_l(rhs.0, self)
    }
}

/* ========= Test ========= */
#[cfg(test)]
mod tests {
    use super::*;
    use cyclotomic_rings::rings::GoldilocksRingNTT;
    use stark_rings::Ring;

    fn demo<R: Ring>() {
        with_cs::<R, _, _>(|cs| {
            let l1 = cs.alloc(R::from(1u128));
            let l2 = cs.alloc(R::from(2u128));

            // L + L -> L
            let l = l1 + l2;

            // L * L -> Q
            let q = l * l1;

            // Q + L -> Q
            let q = q + l;

            // Q * L -> Q
            let q = q * l;

            // Q + Q -> Q
            let q = q + q;

            // Q * Q -> Q
            let _q = q * q;

            // reduce がテープに値を出力しているはず
            // assert!(!cs.view_w().is_empty());
        });
    }

    #[test]
    fn demo_with_branched_list() {
        demo::<GoldilocksRingNTT>()
    }
}

// if x.l++y.lの非ゼロの個数が2Nを超えたらreduceしてwitnessを割り当ててしまう
// その場合、二回目以降のloopのwitnessの割り当て順をどうするか？
// 最適化時にシャッフルしないでshrinkだけすれば一致するはず
