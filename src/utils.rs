use crate::{List, N};
use num_traits::{One, Zero};
use std::collections::HashMap;

/// 2つの疎ベクトル（固定長）を結合して、同じ index を合算し、0 を除去
pub fn merge_and_prune<T>(a: &List<T>, b: &List<T>) -> Vec<(usize, T)>
where
    // T: Copy + Add<Output = T> + PartialEq + Default + Zero,
    T: Copy + Default + PartialEq + One + Zero,
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
pub fn pack_list<T>(v: &[(usize, T)]) -> Result<List<T>, ()>
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
