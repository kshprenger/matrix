use std::{collections::VecDeque, ops::Index, rc::Rc};

use simulator::ProcessId;

pub type VertexPtr = Rc<Vertex>;
type Round = Vec<Option<VertexPtr>>;

pub fn SameVertex(v: &VertexPtr, u: &VertexPtr) -> bool {
    Rc::ptr_eq(v, u)
}

#[derive(PartialEq, Eq, Hash)] // Hashing for fast lookup in buffers
pub struct Vertex {
    pub round: usize,
    pub source: ProcessId,
    pub strong_edges: Vec<VertexPtr>,
}

pub struct RoundBasedDAG {
    matrix: Vec<Round>,
    visited: Vec<Vec<bool>>, // Optimized allocations & constant lookup for iterated bfs
}

impl RoundBasedDAG {
    pub fn New() -> Self {
        Self {
            matrix: Vec::new(),
            visited: Vec::new(),
        }
    }

    pub fn Init(&mut self, n: usize) {
        let genesis_vertices = (0..n)
            .map(|_| Vertex {
                round: 0,
                source: 0,
                strong_edges: Vec::new(),
            })
            .map(|v| Some(VertexPtr::new(v)))
            .collect::<Round>();

        self.matrix.push(genesis_vertices);
        self.visited
            .push((0..n).map(|_| false).collect::<Vec<bool>>());
    }

    // v & u should be already in the DAG
    pub fn PathExists(&mut self, v: &VertexPtr, u: &VertexPtr) -> bool {
        if SameVertex(&v, &u) {
            return true;
        }

        self.ResetVisited();
        self.visited[v.round][v.source] = true;

        let mut queue = VecDeque::new();
        queue.push_back(v);

        while queue.len() > 0 {
            let curr = queue.pop_front().unwrap();
            for edge in &curr.strong_edges {
                if SameVertex(edge, &u) {
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

    pub fn AddVertex(&mut self, v: VertexPtr) {
        if self.matrix.len() > v.round {
            self.Insert(v);
        } else {
            let need_allocate_rounds = self.matrix.len() - v.round + 1;
            self.Grow(need_allocate_rounds);
            self.Insert(v)
        }
    }

    pub fn CurrentAllocatedRounds(&self) -> usize {
        self.matrix.len()
    }

    pub fn CurrentMaxAllocatedRound(&self) -> usize {
        self.CurrentAllocatedRounds() - 1
    }
}

impl RoundBasedDAG {
    fn Grow(&mut self, rounds: usize) {
        let n = self.matrix[0].len();
        (0..rounds).for_each(|_| {
            let mut round = Round::new();
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

impl Index<usize> for RoundBasedDAG {
    type Output = Round;

    fn index(&self, index: usize) -> &Self::Output {
        &self.matrix[index]
    }
}
