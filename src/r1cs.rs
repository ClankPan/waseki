use num_traits::{One, Zero};
use std::collections::{HashMap, hash_map::Entry};
use std::ops::{Add, Mul};

use crate::ar::{Arena, Exp};

type M<T> = HashMap<usize, T>;

/// 入口：ゼロ掃除 → 線形化(A=∅ or B=∅) → 線形定義の連鎖解消 → 代入 → 未使用定義の削除
pub fn optimize<T>(ar: Arena<T>)
where
    T: Copy + One + Zero + PartialEq,
{
    let (_wit, mut exp) = ar.into_inner();

    println!("exp len: {}", exp.len());

    // 1) 0の削除
    println!("1. reduce zero");
    exp.iter_mut().for_each(|(a, b, c, _)| {
        clean_zero(a);
        clean_zero(b);
        clean_zero(c);
    });

    // 2) 積の線形化（片側が {0:c} だけなら C に吸収）
    println!("2. linearize");
    exp.iter_mut().for_each(|(a, b, c, _)| {
        if let Some(l) = linearize(a, b) {
            merge_maps(c, &l);
            a.clear();
            b.clear();
        }
    });

    // 3) 自己参照ガード（右辺 i が A/B/C に出ていないか）
    println!("3. check recursive");
    exp.iter().for_each(|(a, b, c, i)| {
        if let Some(i) = i {
            let bad = a.contains_key(i) || b.contains_key(i) || c.contains_key(i);
            assert!(!bad, "recursive");
        }
    });

    // 4) 線形定義の収集（A=∅, B=∅）
    println!("4. collect linears");
    let mut linears: HashMap<usize, M<T>> = HashMap::new();
    for (a, b, c, i) in &exp {
        if a.is_empty() && b.is_empty() {
            if let Some(i) = i {
                linears.insert(*i, c.clone());
            }
        }
    }
    inline_linears(&mut linears);

    // 5) 収集した線形定義を全制約の A/B/C に代入
    println!("5. inline linears");
    for (a, b, c, _) in exp.iter_mut() {
        apply_subst(a, &linears);
        apply_subst(b, &linears);
        apply_subst(c, &linears);
        clean_zero(a);
        clean_zero(b);
        clean_zero(c);
    }

    // 6) 一度も参照されない「定義専用」制約を削除
    println!("6. delete used");
    let used = {
        let mut s = std::collections::HashSet::new();
        for (a, b, c, _) in &exp {
            s.extend(a.keys().copied());
            s.extend(b.keys().copied());
            s.extend(c.keys().copied());
        }
        s
    };
    exp.retain(|(a, b, _c, i)| match i {
        Some(id) if a.is_empty() && b.is_empty() && !used.contains(id) => false,
        _ => true,
    });

    // 6) 重複制約の削除
    println!("7. dedup exp");
    let exp = dedup_exp(exp);

    println!("optimized exp len: {}", exp.len());
}

/// A または B のどちらかが {0:c} だけなら、もう片方を c 倍して返す
fn linearize<T>(a: &M<T>, b: &M<T>) -> Option<M<T>>
where
    T: Copy + Zero + Mul<Output = T>,
{
    let only_const = |m: &M<T>| (m.len() == 1).then(|| m.get(&0).copied()).flatten();

    match (only_const(a), only_const(b)) {
        (Some(c), None) => Some(scale_map(b, c)),
        (None, Some(c)) => Some(scale_map(a, c)),
        _ => None,
    }
}

/// m 中の k を defs[k] に展開： m[k]*defs[k] を加算し k を削除（変化が尽きるまで）
fn apply_subst<T>(m: &mut M<T>, defs: &HashMap<usize, M<T>>)
where
    T: Copy + Zero + PartialEq + Add<Output = T> + Mul<Output = T>,
{
    loop {
        let targets: Vec<(usize, T)> = m
            .iter()
            .filter_map(|(&k, &coef)| defs.get(&k).map(|_| (k, coef)))
            .collect();
        if targets.is_empty() {
            break;
        }

        let mut changed = false;
        for (k, coef) in targets {
            if m.remove(&k).is_some() {
                if let Some(def) = defs.get(&k) {
                    for (&ik, &iv) in def {
                        match m.entry(ik) {
                            Entry::Vacant(v) => {
                                let val = iv * coef;
                                if val != T::zero() {
                                    v.insert(val);
                                }
                            }
                            Entry::Occupied(mut o) => {
                                let cur = *o.get();
                                let nxt = cur + iv * coef;
                                if nxt == T::zero() {
                                    o.remove();
                                } else {
                                    *o.get_mut() = nxt;
                                }
                            }
                        }
                    }
                }
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

/// 線形定義の連鎖を解消（k→…→tgt の代入を同時更新で収束まで）
fn inline_linears<T>(linears: &mut HashMap<usize, M<T>>)
where
    T: Copy + Zero + PartialEq + Add<Output = T> + Mul<Output = T>,
{
    loop {
        let mut changed = false;
        let mut next = linears.clone();

        for (&k, v) in linears.iter() {
            for (tgt_k, m) in next.iter_mut() {
                if *tgt_k == k {
                    continue; // 自己代入回避
                }
                if let Some(coeff) = m.remove(&k) {
                    for (&ik, &iv) in v {
                        match m.entry(ik) {
                            Entry::Vacant(ent) => {
                                let val = iv * coeff;
                                if val != T::zero() {
                                    ent.insert(val);
                                }
                            }
                            Entry::Occupied(mut ent) => {
                                let cur = *ent.get();
                                let nxt = cur + iv * coeff;
                                if nxt == T::zero() {
                                    ent.remove();
                                } else {
                                    *ent.get_mut() = nxt;
                                }
                            }
                        }
                    }
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
        *linears = next;
    }
}

/// b を a に加算（0 は保持しない）
pub fn merge_maps<T>(a: &mut M<T>, b: &M<T>)
where
    T: Copy + Zero + PartialEq + Add<Output = T>,
{
    for (&k, &vb) in b {
        match a.entry(k) {
            Entry::Vacant(v) => {
                if vb != T::zero() {
                    v.insert(vb);
                }
            }
            Entry::Occupied(mut o) => {
                let cur = *o.get();
                let nxt = cur + vb;
                if nxt == T::zero() {
                    o.remove();
                } else {
                    *o.get_mut() = nxt;
                }
            }
        }
    }
}

/// m を c 倍した新しい M を返す
fn scale_map<T>(m: &M<T>, c: T) -> M<T>
where
    T: Copy + Mul<Output = T>,
{
    let mut out = M::with_capacity(m.len());
    for (&k, &v) in m {
        out.insert(k, v * c);
    }
    out
}

/// 0項を削除
pub fn clean_zero<T>(m: &mut M<T>)
where
    T: Zero + PartialEq,
{
    m.retain(|_, v| !v.is_zero());
}

/// HashMap -> (id昇順) Vec にする（係数はそのまま、0掃除は別）
fn m_to_sorted_vec<T: Copy>(m: &M<T>) -> Vec<(usize, T)> {
    let mut v: Vec<(usize, T)> = m.iter().map(|(&k, &c)| (k, c)).collect();
    v.sort_by_key(|(k, _)| *k);
    v
}

/// (A,B) の順序規約化：まずキー列（id列）で辞書順比較して小さい方を A 側に。
/// 係数の順序付けは不要（id列が違えば順序は一意。id列が同じなら A<->B どちらでも等価）。
fn canonicalize_ab_by_ids<T: Copy>(a: &M<T>, b: &M<T>) -> (Vec<(usize, T)>, Vec<(usize, T)>) {
    let va = m_to_sorted_vec(a);
    let vb = m_to_sorted_vec(b);
    // id 列だけ取り出して比較
    let ids_a: Vec<usize> = va.iter().map(|(k, _)| *k).collect();
    let ids_b: Vec<usize> = vb.iter().map(|(k, _)| *k).collect();
    if (ids_a.len(), &ids_a) <= (ids_b.len(), &ids_b) {
        (va, vb)
    } else {
        (vb, va)
    }
}

#[derive(Clone, PartialEq)]
struct CKey<T: Copy + PartialEq> {
    a: Vec<(usize, T)>, // 既に (A,B) 規約化済み
    b: Vec<(usize, T)>,
    c: Vec<(usize, T)>, // id昇順
    d: Option<usize>,   // 右辺 witness
}

fn make_key<T: Copy + PartialEq>(a: &M<T>, b: &M<T>, c: &M<T>, d: &Option<usize>) -> CKey<T> {
    let (a1, b1) = canonicalize_ab_by_ids(a, b);
    let c1 = m_to_sorted_vec(c);
    CKey {
        a: a1,
        b: b1,
        c: c1,
        d: *d,
    }
}

fn dedup_exp<T>(exp: Vec<Exp<T>>) -> Vec<Exp<T>>
where
    T: Copy + One + Zero + PartialEq,
{
    // まず各制約を正規化（0掃除は既に実施済み想定）
    // 規約化は「キー生成」時に行うのでここでは不要

    let mut seen: Vec<CKey<T>> = Vec::new();
    let mut out = Vec::with_capacity(exp.len());

    for (mut a, mut b, mut c, d) in exp.into_iter() {
        // 0掃除（保険）
        clean_zero(&mut a);
        clean_zero(&mut b);
        clean_zero(&mut c);

        let key = make_key(&a, &b, &c, &d);

        // 既出か判定（PartialEq の線形検索）
        if seen.iter().any(|k| *k == key) {
            // 重複 → 落とす
            continue;
        } else {
            seen.push(key);
            // (A,B) の順序規約化を実データにも反映させる
            let (a_norm, b_norm) = canonicalize_ab_by_ids(&a, &b);
            // Vec→Map へ戻す（必要であれば。Map のままでも意味は同じ）
            let mut a_m = M::with_capacity(a_norm.len());
            for (k, v) in a_norm {
                a_m.insert(k, v);
            }
            let mut b_m = M::with_capacity(b_norm.len());
            for (k, v) in b_norm {
                b_m.insert(k, v);
            }

            // C は並び替え不要（Map は無順序）
            out.push((a_m, b_m, c, d));
        }
    }

    out
}
