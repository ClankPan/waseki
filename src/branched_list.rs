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

pub struct ListIter {

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
        List { tail: b.tail, queue: a.queue, }
    }

}

