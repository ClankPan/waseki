use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Neg,
};

use crate::ar::{Arena, Exp, M};
use num_traits::{One, Zero};

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint<T> {
    /// The A linear combination
    pub a: M<T>,
    /// The B linear combination
    pub b: M<T>,
    /// The C linear combination
    pub c: M<T>,
}

pub struct R1CS<T> {
    /// The number of public inputs
    pub ninputs: usize,
    /// The number of private inputs (auxiliary variables)
    pub nauxs: usize,
    /// The constraints in the system
    pub constraints: Vec<Constraint<T>>,
    //
    pub table: HashSet<usize, usize>,
}

pub fn compile<T>(ar: Arena<T>) -> R1CS<T>
where
    T: Copy + One + Zero + PartialEq + std::fmt::Debug + Neg<Output = T>,
{
    let (wit, mut alloc, mut equal, io) = ar.into_inner();

    // 途中で生成された冗長な制約をなくす
    optimize(&wit, &mut alloc, &io);

    // equalに集約する
    let minus = T::one().neg();
    for (idx, exp) in alloc {
        let exp = match exp {
            Exp::L(mut l) => {
                l.insert(idx, minus);
                Exp::L(l)
            }
            Exp::Q(a, b, mut c) => {
                c.insert(idx, minus);
                Exp::Q(a, b, c)
            }
        };
        equal.push(exp)
    }

    // idxを変換する

    // witnessをそれに沿って削る

    todo!()
}

pub fn optimize<T>(wit: &Vec<T>, alloc: &mut HashMap<usize, Exp<T>>, io: &HashSet<usize>)
where
    T: Copy + One + Zero + PartialEq + std::fmt::Debug,
{
    // 到達集合と frontier を I/O で初期化
    let mut reached: HashSet<usize> = io.clone();
    let mut q: VecDeque<usize> = io.iter().copied().collect();

    while let Some(idx) = q.pop_front() {
        match (alloc.get(&idx), wit.get(idx)) {
            // 線形定義: 参照キーを辿る
            (Some(Exp::L(l)), _) => {
                for &k in l.keys() {
                    if reached.insert(k) {
                        q.push_back(k);
                    }
                }
            }
            // 乗算を含む定義: 参照キーを辿る
            (Some(Exp::Q(a, b, c)), _) => {
                for m in [&a, &b, &c] {
                    for &k in m.keys() {
                        if reached.insert(k) {
                            q.push_back(k);
                        }
                    }
                }
            }
            // alloc には定義が無いが witness はある（葉）→ 何もしない
            (None, Some(_w)) => {}
            // どちらにも無い id は不整合
            (None, None) => panic!("missing idx in alloc and wit: {}", idx),
        }
    }

    // 到達しなかった定義（どこからも使われない式）を削除
    alloc.retain(|id, _| reached.contains(id));
}
