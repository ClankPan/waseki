#[derive(Debug)]
pub struct BranchedList<T> {
    nodes: Vec<Node<T>>,
}

#[derive(Debug)]
pub struct Node<T> {
    value: T,
    next: Option<usize>, // append-only: 最初に一度だけ張る
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    start: usize, // 既定 next に沿って
    len: usize,   // len 個の値を読む
}

#[derive(Debug, Clone)]
pub struct List {
    tail: usize,        // 末尾ノードID（tail.next が未設定なら append 時に張る）
    queue: Vec<Branch>, // 先頭から順に消費する区間列（非空を想定）
}

impl List {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl<T> BranchedList<T> {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn empty_list() -> List {
        List {
            tail: 0,
            queue: Vec::new(),
        } // tail は使わない
    }

    /// 要素1個の List を作る（不変：常に非空）
    pub fn make(&mut self, value: T) -> List {
        let index = self.nodes.len();
        self.nodes.push(Node { value, next: None });
        List {
            tail: index,
            queue: vec![Branch {
                start: index,
                len: 1,
            }],
        }
    }

    /// 既存 list の末尾に value を 1 個追加（= list ++ [value]）
    pub fn push(&mut self, list: List, value: T) -> List {
        let one = self.make(value);
        self.append(list, one)
    }

    /// スライスを末尾に追加（テスト補助）
    pub fn push_slice(&mut self, mut list: List, xs: impl IntoIterator<Item = T>) -> List {
        for x in xs {
            list = self.push(list, x);
        }
        list
    }

    pub fn from_slice(&mut self, xs: &[T]) -> List
    where
        T: Clone,
    {
        if xs.is_empty() {
            return BranchedList::<T>::empty_list();
        }
        let mut it = xs.iter().cloned();
        let mut l = self.make(it.next().unwrap());
        for v in it {
            l = self.push(l, v);
        }
        l
    }

    /// 連結：a ++ b
    ///
    /// 仕様：
    /// - a.tail.next が未設定なら、a.tail.next を「b 先頭ブランチの start」に張る（初回のみ）
    /// - そのうえで a の最後のブランチと b の最初のブランチを結合（start を a 側に、len を加算）
    /// - 残りのブランチは順に連結
    pub fn append(&mut self, mut a: List, mut b: List) -> List {
        if a.queue.is_empty() {
            return b;
        } // ∅ ++ b = b
        if b.queue.is_empty() {
            return a;
        } // a ++ ∅ = a

        debug_assert!(
            !a.queue.is_empty() && !b.queue.is_empty(),
            "List must be non-empty"
        );

        if self.nodes[a.tail].next.is_none() {
            // a の最後の区間と b の最初の区間を圧縮して１区間に
            let a_last = a
                .queue
                .pop()
                .expect("non-empty invariant: a.queue.pop() must succeed");
            let b_first = b
                .queue
                .first_mut()
                .expect("non-empty invariant: b.queue.first_mut() must succeed");

            // 背骨を「初回だけ」張る（append-only）
            self.nodes[a.tail].next = Some(b_first.start);

            // 区間の結合：b 先頭を a_last に吸収
            b_first.start = a_last.start;
            b_first.len += a_last.len;
        } else {
            // もし既に next がある場合、ここで a_last と b_first を強制結合しない方針もあり。
            // 今回は「next が既にある＝a は背骨済みの経路」と見なし、区間列を単純連結。
        }

        a.queue.extend(b.queue);
        List {
            tail: b.tail,
            queue: a.queue,
        }
    }

    /// list を読むためのイテレータ（`&T` を順に返す）
    pub fn iter<'a>(&'a self, list: &List) -> ListIter<'a, T> {
        ListIter {
            nodes: &self.nodes,
            branches: list.queue.clone(),
            br_ix: 0,
            cur: None,
            rem: 0,
        }
    }
}

pub struct ListIter<'a, T> {
    nodes: &'a [Node<T>],
    branches: Vec<Branch>, // 0番目から順に消費
    // 内部状態
    br_ix: usize,       // 現在の branch インデックス
    cur: Option<usize>, // 現在ノードID（同一branch内で前進）
    rem: usize,         // 現在branchで残り要素数
}

impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // 残数が0なら次のbranchへ（0長branchはスキップ）
        while self.rem == 0 {
            if self.br_ix >= self.branches.len() {
                return None;
            }
            let br = &self.branches[self.br_ix];
            self.cur = Some(br.start);
            self.rem = br.len;
            self.br_ix += 1;
            if self.rem == 0 {
                continue; // 0長ならスキップして次へ
            }
        }

        // 同一branch内の1ステップ
        let u = match self.cur {
            Some(u) => u,
            None => return None, // 不整合：next が足りない等（デバッグ時に気づく）
        };
        let node = &self.nodes[u];
        let out: &T = &node.value;

        // 次の位置へ前進
        if self.rem > 1 {
            // 既定 next で前進
            self.cur = node.next;
            debug_assert!(
                self.cur.is_some(),
                "branch length exceeds the available next-links"
            );
            self.rem -= 1;

            // 優しくするなら（nextがNoneでも落とさない）:
            if self.cur.is_none() {
                self.rem = 0; // このbranchをここで打ち切る
            }
        } else {
            // この要素でbranch終了
            self.rem = 0;
        }

        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_single_and_iter() {
        let mut bl = BranchedList::new();
        let l = bl.make(42);

        let got: Vec<_> = bl.iter(&l).copied().collect();
        assert_eq!(got, vec![42]);
    }

    #[test]
    fn push_chain_values() {
        let mut bl = BranchedList::new();
        let l = bl.make(1);
        let l = bl.push(l, 2);
        let l = bl.push(l, 3);
        let got: Vec<_> = bl.iter(&l).copied().collect();
        assert_eq!(got, vec![1, 2, 3]); // 直鎖
    }

    #[test]
    fn append_two_single_lists() {
        let mut bl = BranchedList::new();
        let a = bl.make(10);
        let b = bl.make(20);

        let c = bl.append(a, b);
        let got: Vec<_> = bl.iter(&c).copied().collect();
        assert_eq!(got, vec![10, 20]);
    }

    #[test]
    fn append_is_associative_over_values() {
        // (a ++ b) ++ c と a ++ (b ++ c) で値列が等しいことを確認
        let mut bl = BranchedList::new();
        let a = bl.make(1);
        let b = bl.make(2);
        let c = bl.make(3);

        let ab = bl.append(a, b);
        let abc1 = bl.append(ab, c);

        let mut bl2 = BranchedList::new();
        let a2 = bl2.make(1);
        let b2 = bl2.make(2);
        let c2 = bl2.make(3);
        let bc = bl2.append(b2, c2);
        let abc2 = bl2.append(a2, bc);

        let got1: Vec<_> = bl.iter(&abc1).copied().collect();
        let got2: Vec<_> = bl2.iter(&abc2).copied().collect();
        assert_eq!(got1, got2);
        assert_eq!(got1, vec![1, 2, 3]);
    }

    #[test]
    fn append_compresses_boundary_branch() {
        // a.last_branch と b.first_branch が 1 つの区間に圧縮されることを確認
        let mut bl = BranchedList::new();
        let a = bl.make(1);
        let a = bl.push_slice(a, [2, 3]); // [1,2,3]
        let b = bl.make(4);
        let b = bl.push_slice(b, [5]); // [4,5]
        let c = bl.append(a, b);

        // c.queue の最初のブランチは [1,2,3,4,5] の 5 要素区間になっているはず
        // （実装詳細に踏み込むテスト：queue を直接見る）
        assert!(!c.queue.is_empty());
        let first = &c.queue[0];
        assert_eq!(first.len, 5);

        let got: Vec<_> = bl.iter(&c).copied().collect();
        assert_eq!(got, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn append_compresses_boundary_branch_with_3_branches() {
        // a.last_branch と b.first_branch が 1 つの区間に圧縮されることを確認
        let mut bl = BranchedList::new();
        let a = bl.make(1);
        let a = bl.push_slice(a, [2, 3]); // [1,2,3]
        let b = bl.make(4);
        let b = bl.push_slice(b, [5]); // [4,5]
        let c = bl.make(5);
        let _ = bl.append(a.clone(), c);
        let d = bl.append(a, b);

        // d.queue の最初のブランチは [1,2,3] の 3 要素区間になっているはず
        // なぜなら3の要素のnextには、5が繋がれているから
        assert!(!d.queue.is_empty());
        let first = &d.queue[0];
        assert_eq!(first.len, 3);

        let got: Vec<_> = bl.iter(&d).copied().collect();
        assert_eq!(got, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn backbone_is_set_only_once() {
        // a.tail.next は初回 append だけで設定され、2 回目以降は上書きしない方針
        let mut bl = BranchedList::new();
        let base = bl.make(1);
        let a = bl.push_slice(base, [2]); // [1,2]
        let b = bl.make(3); // [3]
        let c = bl.make(4); // [4]

        let ab = bl.append(a, b); // ここで a.tail.next が初回設定
        let abc = bl.append(ab, c); // 2 回目は next 既設 → 圧縮は発生しないが値列は正しい

        let got: Vec<_> = bl.iter(&abc).copied().collect();
        assert_eq!(got, vec![1, 2, 3, 4]);
    }

    #[test]
    fn iter_empty_never_happens_but_zero_len_branch_is_skipped() {
        // 設計上 List は非空だが、万一 len=0 の branch が入っていても落ちない（スキップ）
        let mut bl = BranchedList::new();
        let base = bl.make(1);
        let mut l = bl.push_slice(base, [2, 3]); // [1,2,3]
        // 不正に 0 長 branch を挿入（※通常APIでは作れない）
        l.queue.insert(
            0,
            Branch {
                start: l.tail,
                len: 0,
            },
        );

        let got: Vec<_> = bl.iter(&l).copied().collect();
        assert_eq!(got, vec![1, 2, 3]);
    }
}
