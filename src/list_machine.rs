use std::ops::{Mul, MulAssign};

#[derive(Debug)]
pub struct Machine<T> {
    list: Vec<Node<T>>,
}

#[derive(Debug)]
pub struct Node<T> {
    wit: usize,
    coeff: T,
    next: Option<usize>, // append-only: 最初に一度だけ張る
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    start: usize, // 既定 next に沿って
    len: usize,   // len 個の値を読む
}

#[derive(Debug)]
pub enum Op<T> {
    Br(Branch),
    Mul(T),
}

#[derive(Copy, Clone)]
pub struct AAA {
    aaa: [u32; 10000],
    op: Option<u32>,
}

#[derive(Debug)]
pub struct Code<T> {
    head: usize,
    tail: usize,
    code: Vec<Op<T>>,
}

impl<T: Default> Machine<T>
where
    T: MulAssign<T>,
{
    pub fn make(&mut self, wit: usize) -> Code<T> {
        let index = self.list.len();
        self.list.push(Node {
            wit,
            coeff: T::default(),
            next: None,
        });
        Code {
            head: index,
            tail: index,
            code: vec![Op::Br(Branch {
                start: index,
                len: 1,
            })],
        }
    }

    pub fn append(&mut self, mut a: Code<T>, mut b: Code<T>) -> Code<T> {
        if a.code.is_empty() {
            return b;
        }
        if b.code.is_empty() {
            return a;
        }

        if self.list[a.tail].next.is_none() {
            // a の最後の区間と b の最初の区間を圧縮して１区間に
            let a_last = a
                .code
                .pop()
                .expect("non-empty invariant: a.queue.pop() must succeed");
            let b_first = b
                .code
                .first_mut()
                .expect("non-empty invariant: b.queue.first_mut() must succeed");

            self.list[a.tail].next = Some(b.head);

            match (a_last, b_first) {
                (Op::Br(a_br), Op::Br(b_br)) => {
                    // Merge
                    b_br.start = a_br.start;
                    b_br.len += a_br.len;
                }
                (Op::Br(br), Op::Mul(coeff)) => {
                    // if br.len == 1 {
                    // } else {
                    //     a.code.push(Op::Br(br)) // 戻す
                    // }
                    a.code.push(Op::Br(br)) // 戻す
                }
                (Op::Mul(c), Op::Br(_)) => a.code.push(Op::Mul(c)),
                (Op::Mul(coeff_a), Op::Mul(coeff_b)) => {
                    // Merge
                    *coeff_b *= coeff_a;
                }
            }

            // 区間の結合：b 先頭を a_last に吸収
            // b_first.start = a_last.start;
            // b_first.len += a_last.len;
        } else {
            // もし既に next がある場合、ここで a_last と b_first を強制結合しない方針もあり。
            // 今回は「next が既にある＝a は背骨済みの経路」と見なし、区間列を単純連結。
        }

        a.code.extend(b.code);
        Code {
            head: a.head,
            tail: b.tail,
            code: a.code,
        }
    }
}
