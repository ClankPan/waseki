use std::{
    collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
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
    pub table: HashMap<usize, usize>,
}

impl<T> R1CS<T>
where
    T: Copy + One + Zero + PartialEq + std::fmt::Debug + Neg<Output = T>,
{
    pub fn witness(&self, auxes: Vec<T>) -> Vec<T> {
        build_witness(auxes, &self.table)
    }
}

pub fn compile<T>(ar: Arena<T>) -> R1CS<T>
where
    T: Copy + One + Zero + PartialEq + std::fmt::Debug + Neg<Output = T>,
{
    let (auxes, mut wires, exprs, io) = ar.into_inner();

    let ninputs = io.len();
    // 途中で生成された冗長な制約をなくす
    optimize(&auxes, &mut wires, &io);

    let (constraints, table) = build_constraints(wires, exprs, io);

    R1CS {
        ninputs,
        nauxs: table.len() - ninputs,
        constraints,
        table,
    }
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

pub fn build_witness<T>(auxes: Vec<T>, table: &HashMap<usize, usize>) -> Vec<T>
where
    T: Copy + One + Zero + PartialEq + std::fmt::Debug + Neg<Output = T>,
{
    let mut witness = vec![T::zero(); table.len()];
    for (a, b) in table {
        witness[*b] = auxes[*a]
    }
    witness
}
pub fn build_constraints<T>(
    wires: HashMap<usize, Exp<T>>,
    mut exprs: Vec<Exp<T>>,
    io: HashSet<usize>,
) -> (Vec<Constraint<T>>, HashMap<usize, usize>)
where
    T: Copy + One + Zero + PartialEq + std::fmt::Debug + Neg<Output = T>,
{
    // equalに集約する
    let minus = T::one().neg();
    for (idx, exp) in wires {
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
        exprs.push(exp)
    }

    // 1) idx を変換するテーブルを作成（I/O は先頭に固定割当）
    let mut table: HashMap<usize, usize> =
        io.into_iter().enumerate().map(|(i, id)| (id, i)).collect();

    // 2) equal に現れる id をすべてインターン
    for exp in &exprs {
        match exp {
            Exp::L(l) => intern_keys(&mut table, l),
            Exp::Q(a, b, c) => {
                intern_keys(&mut table, a);
                intern_keys(&mut table, b);
                intern_keys(&mut table, c);
            }
        }
    }

    // 3) equal → constraints（キー写像しながら変換）
    let mut constraints = Vec::with_capacity(exprs.len());
    for exp in exprs.drain(..) {
        constraints.push(exp_to_constraint(exp, &table));
    }

    (constraints, table)
}

// equal 内の各マップに現れる id を table に登録（未登録なら連番を割当）
fn intern_keys<T>(table: &mut HashMap<usize, usize>, m: &M<T>) {
    for &id in m.keys() {
        let idx = table.len();
        if let Entry::Vacant(v) = table.entry(id) {
            v.insert(idx);
        }
    }
}

// m のキーを table に従って写像（消費版）
fn remap_map<T>(m: M<T>, table: &HashMap<usize, usize>) -> M<T> {
    let mut out = M::with_capacity(m.len());
    for (id, v) in m {
        let k = *table.get(&id).expect("id must be interned");
        out.insert(k, v);
    }
    out
}

// 1つの Exp から Constraint へ（キー写像込み）
fn exp_to_constraint<T>(exp: Exp<T>, table: &HashMap<usize, usize>) -> Constraint<T> {
    match exp {
        Exp::L(l) => Constraint {
            a: HashMap::new(),
            b: HashMap::new(),
            c: remap_map(l, table),
        },
        Exp::Q(a, b, c) => Constraint {
            a: remap_map(a, table),
            b: remap_map(b, table),
            c: remap_map(c, table),
        },
    }
}
