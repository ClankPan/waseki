mod branched_list;
mod list_machine;

use num_traits::{One, Zero};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Add, Mul};

pub struct Arena<T> {
    wit: RefCell<Vec<T>>,
    exp: RefCell<Vec<(Vec<(usize, T)>, Vec<(usize, T)>, Vec<(usize, T)>, usize)>>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            wit: RefCell::new(Vec::new()),
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

impl<'id, T: Default + Copy> L<'id, T> {
    #[inline]
    fn new(ar: &'id Arena<T>) -> Self {
        Self {
            v: T::default(),
            l: [(0, T::default()); N],
            ar,
        }
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

/* ========= デモ ========= */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_with_branched_list() {
        with_cs::<i32, _, _>(|cs| {
            // // L を 2 本
            let l1 = cs.alloc(1);
            let l2 = cs.alloc(2);

            // L + L -> L
            let l = l1 + l2;

            // L * L -> Q
            let q = l * l1;

            // Q + L -> Q
            let q = q + l;

            // Q * Q -> Q
            let _q = q * (l1 * l2);

            // reduce がテープに値を出力しているはず
            // assert!(!cs.view_w().is_empty());
        });
    }
}

// if x.l++y.lの非ゼロの個数が2Nを超えたらreduceしてwitnessを割り当ててしまう
// その場合、二回目以降のloopのwitnessの割り当て順をどうするか？
// 最適化時にシャッフルしないでshrinkだけすれば一致するはず

//
// #[inline]
// fn l_mul_l<'id, T: Clone>(x: L<'id, T>, y: L<'id, T>) -> Q<'id, T>
// where
//     T: Mul<Output = T>,
// {
//     debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
//     Q {
//         a: x.l,
//         b: y.l,
//         c: x.ar.empty_list(),
//         ar: x.ar,
//         v: x.v * y.v,
//     }
// }

// /// ========== Arena（BranchedList ベース） ==========
// struct Arena<T> {
//     bl: RefCell<BranchedList<T>>, // 背骨 + 区間列（append-only）
//     lists: RefCell<Vec<List>>,    // List をインデックスで管理
//     q_nodes: RefCell<Vec<QNode>>, // Q の中身（Listインデックスで保持）
//     w: RefCell<Vec<T>>,           // 例: R1CSテープ（ここでは値をそのまま積む）
// }
//
// impl<T> Default for Arena<T> {
//     fn default() -> Self {
//         Self {
//             bl: RefCell::new(BranchedList::new()),
//             lists: RefCell::new(Vec::new()),
//             q_nodes: RefCell::new(Vec::new()),
//             w: RefCell::new(Vec::new()),
//         }
//     }
// }
//
// #[derive(Clone, Debug, PartialEq, Eq)]
// struct QNode {
//     a: u32,         // lists[a]
//     b: u32,         // lists[b]
//     c: Option<u32>, // 追加の線形（無いこともある）
// }
//
// impl<T> Arena<T> {
//     /* ------ List（L）側のヘルパ ------ */
//
//     /// Vec<T> から 1 本の List を作って登録、インデックスを返す
//     #[inline]
//     fn list_from_vec(&self, data: Vec<T>) -> u32 {
//         if data.is_empty() {
//             // 空も許容したい場合はこれ（不要なら expect に戻してください）
//             return self.empty_list();
//         }
//
//         let mut bl = self.bl.borrow_mut();
//         let mut it = data.into_iter();
//         let first = it.next().unwrap(); // data は非空
//
//         // 先頭だけ make し、残りは push_slice でまとめて追加
//         let base = bl.make(first);
//         let lst = bl.push_slice(base, it); // ← ここで Iterator をそのまま渡せる
//
//         self.push_list(lst)
//     }
//
//     /// 既存 List を登録してインデックスを返す
//     #[inline]
//     fn push_list(&self, list: List) -> u32 {
//         let mut ls = self.lists.borrow_mut();
//         let id = ls.len() as u32;
//         ls.push(list);
//         id
//     }
//
//     /// List をクローンで取り出す（内部では所有）
//     #[inline]
//     fn get_list(&self, idx: u32) -> List {
//         self.lists.borrow()[idx as usize].clone()
//     }
//
//     /// a ++ b を作って登録し、インデックスを返す
//     #[inline]
//     fn append_lists(&self, a_idx: u32, b_idx: u32) -> u32 {
//         let a = self.get_list(a_idx);
//         let b = self.get_list(b_idx);
//         let mut bl = self.bl.borrow_mut();
//         let c = bl.append(a, b);
//         self.push_list(c)
//     }
//
//     /// 空の List を作る（c 用）。必要に応じて 0 長 branch を許容する版。
//     #[inline]
//     fn empty_list(&self) -> u32 {
//         // 空の List を表現したい場合：queue が空の List を直接持つ
//         // tail は未使用だが、無効インデックスを避けるため 0 を入れておく
//         self.push_list(self.br.empty())
//     }
//
//     /* ------ Q 側のヘルパ ------ */
//
//     #[inline]
//     fn push_q(&self, a: u32, b: u32, c: Option<u32>) -> u32 {
//         let mut q = self.q_nodes.borrow_mut();
//         let idx = q.len() as u32;
//         q.push(QNode { a, b, c });
//         idx
//     }
//
//     #[inline]
//     fn get_q(&self, idx: u32) -> QNode {
//         self.q_nodes.borrow()[idx as usize].clone()
//     }
//
//     /* ------ テープ ------ */
//
//     #[inline]
//     fn append_w<I>(&self, iter: I)
//     where
//         I: IntoIterator<Item = T>,
//     {
//         self.w.borrow_mut().extend(iter);
//     }
//     #[inline]
//     fn view_w(&self) -> Ref<'_, Vec<T>> {
//         self.w.borrow()
//     }
// }
//
// /// ========== CS（ブランド付きハブ） ==========
// pub fn with_cs<T, R, F>(f: F) -> R
// where
//     F: for<'id> FnOnce(CS<'id, T>) -> R,
// {
//     let arena = Arena::<T>::default();
//     let cs = CS {
//         ar: &arena,
//         _brand: PhantomData::<&mut ()>,
//     };
//     f(cs)
// }
//
// #[derive(Copy, Clone)]
// pub struct CS<'id, T> {
//     ar: &'id Arena<T>,
//     _brand: PhantomData<&'id mut ()>, // generative brand（不変）
// }
//
// impl<'id, T> CS<'id, T> {
//     #[inline]
//     pub fn view_w(&self) -> Ref<'_, Vec<T>> {
//         self.ar.view_w()
//     }
// }
//
// impl<'id, T: Clone> CS<'id, T> {
//     #[inline]
//     pub fn make_l(&self, data: Vec<T>) -> L<'id, T> {
//         L {
//             l_idx: self.ar.list_from_vec(data),
//             ar: self.ar,
//         }
//     }
//     #[inline]
//     pub fn make_q_from_lists(&self, a: L<'id, T>, b: L<'id, T>, c: Option<L<'id, T>>) -> Q<'id, T> {
//         let c_idx = c.map(|x| x.l_idx);
//         Q {
//             q_idx: self.ar.push_q(a.l_idx, b.l_idx, c_idx),
//             ar: self.ar,
//         }
//     }
// }
//
// /// ========== ハンドル（Copy） ==========
// #[derive(Copy, Clone)]
// pub struct L<'id, T: Clone> {
//     l_idx: u32,
//     ar: &'id Arena<T>,
// }
// #[derive(Copy, Clone)]
// pub struct Q<'id, T: Clone> {
//     q_idx: u32,
//     ar: &'id Arena<T>,
// }
//
// /* ========= コア演算 ========= */
//
// #[inline]
// fn l_add_l<'id, T: Clone>(x: L<'id, T>, y: L<'id, T>) -> L<'id, T> {
//     debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
//     let out = x.ar.append_lists(x.l_idx, y.l_idx);
//     L {
//         l_idx: out,
//         ar: x.ar,
//     }
// }
//
// #[inline]
// fn l_mul_l<'id, T: Clone>(x: L<'id, T>, y: L<'id, T>) -> Q<'id, T> {
//     debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
//     let q_idx = x.ar.push_q(x.l_idx, y.l_idx, None);
//     Q { q_idx, ar: x.ar }
// }
//
// #[inline]
// fn q_add_l<'id, T: Clone>(q: Q<'id, T>, l: L<'id, T>) -> Q<'id, T> {
//     debug_assert!(std::ptr::eq(q.ar as *const _, l.ar as *const _));
//     let QNode { a, b, c } = q.ar.get_q(q.q_idx);
//     let c_idx = match c {
//         Some(c0) => q.ar.append_lists(c0, l.l_idx),
//         None => l.l_idx,
//     };
//     Q {
//         q_idx: q.ar.push_q(a, b, Some(c_idx)),
//         ar: q.ar,
//     }
// }
//
// /// reduce: Q をテープに出力し、(a ++ b ++ c) を L として返す
// #[inline]
// pub fn reduce<'id, T: Clone>(q: Q<'id, T>) -> L<'id, T> {
//     let QNode { a, b, c } = q.ar.get_q(q.q_idx);
//
//     // a, b, c を順にストリーム出力（branched_listの iter を使う）
//     {
//         let bl = q.ar.bl.borrow();
//         // a
//         for v in bl.iter(&q.ar.get_list(a)) {
//             q.ar.append_w(std::iter::once(v.clone()));
//         }
//         // b
//         for v in bl.iter(&q.ar.get_list(b)) {
//             q.ar.append_w(std::iter::once(v.clone()));
//         }
//         // c（あれば）
//         if let Some(cidx) = c {
//             for v in bl.iter(&q.ar.get_list(cidx)) {
//                 q.ar.append_w(std::iter::once(v.clone()));
//             }
//         }
//     }
//
//     // 返り値 L = a ++ b ++ c
//     let ab = q.ar.append_lists(a, b);
//     let abc = if let Some(cidx) = c {
//         q.ar.append_lists(ab, cidx)
//     } else {
//         ab
//     };
//     L {
//         l_idx: abc,
//         ar: q.ar,
//     }
// }
//
// #[inline]
// fn q_mul_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> Q<'id, T> {
//     debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
//     let lx = reduce(x);
//     let ly = reduce(y);
//     l_mul_l(lx, ly) // a=lx, b=ly, c=None
// }
//
// #[inline]
// fn q_add_q<'id, T: Clone>(x: Q<'id, T>, y: Q<'id, T>) -> L<'id, T> {
//     debug_assert!(std::ptr::eq(x.ar as *const _, y.ar as *const _));
//     let lx = reduce(x);
//     let ly = reduce(y);
//     l_add_l(lx, ly)
// }
//
// /* ========= 演算子トレイト（Copyなので owned-owned でOK） ========= */
//
// impl<'id, T: Clone> Add for L<'id, T> {
//     type Output = L<'id, T>;
//     fn add(self, rhs: Self) -> Self::Output {
//         l_add_l(self, rhs)
//     }
// }
// impl<'id, T: Clone> Mul for L<'id, T> {
//     type Output = Q<'id, T>;
//     fn mul(self, rhs: Self) -> Self::Output {
//         l_mul_l(self, rhs)
//     }
// }
// impl<'id, T: Clone> Add<L<'id, T>> for Q<'id, T> {
//     type Output = Q<'id, T>;
//     fn add(self, rhs: L<'id, T>) -> Self::Output {
//         q_add_l(self, rhs)
//     }
// }
// impl<'id, T: Clone> Mul for Q<'id, T> {
//     type Output = Q<'id, T>;
//     fn mul(self, rhs: Self) -> Self::Output {
//         q_mul_q(self, rhs)
//     }
// }
// impl<'id, T: Clone> Add for Q<'id, T> {
//     type Output = L<'id, T>;
//     fn add(self, rhs: Self) -> Self::Output {
//         q_add_q(self, rhs)
//     }
// }
//
// /* ========= デモ ========= */
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn demo_with_branched_list() {
//         with_cs::<i32, _, _>(|cs| {
//             // L を 2 本
//             let l1 = cs.make_l(vec![1, 2]);
//             let l2 = cs.make_l(vec![3]);
//
//             // L + L -> L（BranchedList::append で連結）
//             let l = l1 + l2;
//
//             // L * L -> Q（a=l, b=l, c=None）
//             let q = l * l1;
//
//             // Q + L -> Q（c に L を合流）
//             let q = q + l;
//
//             // Q * Q -> Q（内部で reduce → L*L）
//             let _q = q * (l1 * l2);
//
//             // reduce がテープに値を出力しているはず
//             assert!(!cs.view_w().is_empty());
//         });
//     }
// }
