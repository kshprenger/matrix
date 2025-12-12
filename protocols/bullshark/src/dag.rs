use std::{collections::VecDeque, ops::Index, ptr, rc::Rc};

use simulator::ProcessId;

type VertexPtr = Rc<Vertex>;

fn IsSameVertex(v: &VertexPtr, u: &VertexPtr) -> bool {
    ptr::eq(v.as_ref(), u.as_ref())
}

pub(super) struct Vertex {
    round: usize,
    source: ProcessId,
    strong_edges: Vec<VertexPtr>,
}

pub(super) struct DAG {
    matrix: Vec<Vec<Option<VertexPtr>>>,
    visited: Vec<Vec<bool>>, // Optimized allocations & constant lookup
}

impl DAG {
    pub(super) fn New(n: usize) -> Self {
        let genesis_vertices = (0..n)
            .map(|_| Vertex {
                round: 0,
                source: 0,
                strong_edges: Vec::new(),
            })
            .map(|v| Some(VertexPtr::new(v)))
            .collect::<Vec<Option<VertexPtr>>>();

        let mut matrix = Vec::new();
        matrix.push(genesis_vertices);

        let mut visited = Vec::new();
        visited.push((0..n).map(|_| false).collect::<Vec<bool>>());

        Self { matrix, visited }
    }

    // v & u already in the DAG
    pub(super) fn PathExists(&mut self, v: VertexPtr, u: VertexPtr) -> bool {
        if IsSameVertex(&v, &u) {
            return true;
        }

        self.ResetVisited();
        self.visited[v.round][v.source] = true;

        let mut queue = VecDeque::new();
        queue.push_back(&v);

        while queue.len() > 0 {
            let curr = queue.pop_front().unwrap();
            for edge in &curr.strong_edges {
                if IsSameVertex(edge, &u) {
                    return true;
                } else {
                    if self.visited[edge.round][edge.source] {
                        continue;
                    } else {
                        self.visited[edge.round][edge.source] = true;
                        queue.push_back(edge);
                    }
                }
            }
        }

        return false;
    }

    pub(super) fn Add(&mut self, v: VertexPtr) {
        if self.matrix.len() > v.round {
            self.Insert(v);
        } else {
            let need_allocate_rounds = self.matrix.len() - v.round + 1;
            self.Grow(need_allocate_rounds);
            self.Insert(v)
        }
    }
}

impl DAG {
    fn Grow(&mut self, rounds: usize) {
        let n = self.matrix[0].len();
        (0..rounds).for_each(|_| {
            let mut round = Vec::<Option<VertexPtr>>::new();
            round.resize(n, None);
            let mut round_visited = Vec::new();
            round_visited.resize(n, false);

            self.matrix.push(round);
            self.visited.push(round_visited);
        });
    }

    fn Insert(&mut self, v: VertexPtr) {
        let round = v.round;
        let source = v.source;
        self.matrix[round][source] = Some(v);
    }

    fn ResetVisited(&mut self) {
        self.visited.iter_mut().for_each(|round| {
            let l = round.len();
            round.clear();
            round.resize(l, false);
        });
    }
}

impl Index<usize> for DAG {
    type Output = Vec<Option<VertexPtr>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.matrix[index]
    }
}
