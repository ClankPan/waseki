pub struct BranchedList<T> {
    nodes: Vec<Node<T>>,
}

pub struct Node<T> {
    value: T,
    next: Option<usize>,
}

#[derive(Clone)]
pub struct Branch {
    start: usize,
    len: usize,
}

pub struct List {
    tail: usize,
    queue: Vec<Branch>,
}

impl<T> BranchedList<T> {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn make(&mut self, value: T) -> List {
        let index = self.nodes.len();
        self.nodes.push(Node { value, next: None });
        List {
            tail: index,
            queue: vec![Branch {
                len: 1,
                start: index,
            }],
        }
    }

    pub fn push(&mut self, list: List, value: T) -> List {
        let x = self.make(value);
        self.append(list, x)
    }

    pub fn append(&mut self, mut a: List, mut b: List) -> List {
        if self.nodes[a.tail].next.is_none() {
            let a_last_br = a.queue.pop().unwrap();
            let b_first_br = b.queue.first_mut().unwrap();
            self.nodes[a.tail].next = Some(b_first_br.start);
            b_first_br.start = a_last_br.start;
            b_first_br.len += a_last_br.len;
        }
        a.queue.extend(b.queue);
        List {
            tail: b.tail,
            queue: a.queue,
        }
    }
}

pub struct ListIter<'a, T> {
    nodes: &'a Vec<Node<T>>,
    branches: Vec<Branch>, // 0番目から順に消費
    // 内部状態
    br_ix: usize,       // 現在の branch インデックス
    cur: Option<usize>, // 現在ノードID（同一branch内で前進）
    rem: usize,         // 現在branchで残り要素数
}

// BranchedList 側から作るヘルパ（お好みで）
impl<T> BranchedList<T> {
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
            None => return None, // 不整合（nextが足りない等）
        };
        let node = &self.nodes[u];
        let out: &T = &node.value;

        // 次の位置へ前進
        if self.rem > 1 {
            // 既定nextで前進
            self.cur = node.next;
            // 期待では next は Some。足りない場合は分かりやすく潰すか、優しく終わるかを選ぶ
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
