use std::collections::{HashSet, VecDeque};

use crate::ar::{Arena, Exp};
use num_traits::{One, Zero};

pub fn optimize<T>(ar: Arena<T>)
where
    T: Copy + One + Zero + PartialEq + std::fmt::Debug,
{
    let (wit, mut alloc, _equal, io) = ar.into_inner();

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

    // 必要ならここで exp/wit を作り直す or 返す
}
