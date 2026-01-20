use std::{
    collections::VecDeque,
    ops::Index,
    rc::{Rc, Weak},
};

use matrix::{
    CurrentId, Now, ProcessId,
    global::anykv,
    time::{self},
};

const GC_REMAIN: usize = 40;

pub type VertexPtr = Rc<Vertex>;
type Round = Vec<Option<VertexPtr>>;

pub fn SameVertex(v: &VertexPtr, u: &VertexPtr) -> bool {
    Rc::ptr_eq(v, u)
}

pub struct Vertex {
    pub round: usize,
    pub source: ProcessId,
    pub creation_time: time::Jiffies,

    // Each vertex is a pointer to real one. (Each vertex is allocated exactly-once during execution)
    // Each party contains strong Rc references to vertices in their dags.
    // At the same time in the real dag edges are represented with weak Rc references.
    // Once all parties GC-ed their dags, Vertices will be deallocated because there will be no more strong Rc references.
    // Until GC time is is safe for the process to upgrade Weak refs.
    pub strong_edges: Vec<Weak<Vertex>>,
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        (self.round, self.source).eq(&(other.round, other.source))
    }
    fn ne(&self, other: &Self) -> bool {
        (self.round, self.source).ne(&(other.round, other.source))
    }
}

impl Eq for Vertex {}

impl PartialOrd for Vertex {
    fn ge(&self, other: &Self) -> bool {
        (self.round, self.source).ge(&(other.round, other.source))
    }
    fn le(&self, other: &Self) -> bool {
        (self.round, self.source).le(&(other.round, other.source))
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.round, self.source).partial_cmp(&(other.round, other.source))
    }
}

impl Ord for Vertex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.round, self.source).cmp(&(other.round, other.source))
    }
}

#[derive(Default)]
pub struct RoundBasedDAG {
    proc_num: usize,
    matrix: VecDeque<Round>,
    visited: VecDeque<Vec<bool>>, // Optimized allocations & constant lookup for iterated bfs
    ordered: VecDeque<Vec<bool>>,
    gc_offset: usize,
}

impl RoundBasedDAG {
    pub fn SetRoundSize(&mut self, proc_num: usize) {
        self.proc_num = proc_num;
    }

    // v should be already in the DAG
    // "in some deterministic order"
    pub fn OrderFrom(&mut self, v: &VertexPtr) {
        let mut queue = VecDeque::new();
        queue.push_back(v.clone());

        while queue.len() > 0 {
            let curr = queue.pop_front().unwrap();

            let strong_edges: Vec<VertexPtr> = curr
                .strong_edges
                .iter()
                .map(|weak| weak.upgrade().unwrap())
                .collect();

            for edge in strong_edges.into_iter() {
                let real_round = self.Round(edge.round);
                if self.ordered[real_round][edge.source] {
                    continue;
                } else {
                    self.ordered[real_round][edge.source] = true;
                    if CurrentId() == edge.source {
                        anykv::Modify::<(f64, usize)>(
                            "avg_latency",
                            |(prev_avg_latency, prev_total_ordered)| {
                                let vertex_latency = Now() - edge.creation_time;
                                *prev_avg_latency = (vertex_latency.0 as f64
                                    + (*prev_avg_latency * *prev_total_ordered as f64))
                                    as f64
                                    / (*prev_total_ordered + 1) as f64;

                                *prev_total_ordered += 1;
                            },
                        );
                    }
                    queue.push_back(edge);
                }
            }
        }
        self.GC();
    }

    // v & u should be already in the DAG
    // u.round <= v.round
    pub fn PathExists(&mut self, v: &VertexPtr, u: &VertexPtr) -> bool {
        if SameVertex(&v, &u) {
            return true;
        }

        let read_round = self.Round(v.round);

        self.ResetVisited();
        self.visited[read_round][v.source] = true;

        let mut queue = VecDeque::new();
        queue.push_back(v.clone());

        while queue.len() > 0 {
            let curr = queue.pop_front().unwrap();

            let strong_edges: Vec<VertexPtr> = curr
                .strong_edges
                .iter()
                .map(|weak| weak.upgrade().unwrap())
                .collect();

            for edge in strong_edges.into_iter() {
                // Reached depth
                if edge.round < u.round {
                    continue;
                }
                if SameVertex(&edge, &u) {
                    return true;
                } else {
                    let read_round = self.Round(edge.round);
                    if self.visited[read_round][edge.source] {
                        continue;
                    } else {
                        self.visited[read_round][edge.source] = true;
                        queue.push_back(edge);
                    }
                }
            }
        }

        return false;
    }

    pub fn AddVertex(&mut self, v: VertexPtr) {
        if self.CurrentAllocatedRounds() > v.round {
            self.Insert(v);
        } else {
            let need_allocate_rounds = self.CurrentAllocatedRounds() - v.round + 1;
            self.Grow(need_allocate_rounds);
            self.Insert(v)
        }
    }

    pub fn CurrentAllocatedRounds(&self) -> usize {
        self.matrix.len() + self.gc_offset
    }

    pub fn CurrentMaxAllocatedRound(&self) -> usize {
        self.CurrentAllocatedRounds().saturating_sub(1)
    }
}

impl RoundBasedDAG {
    // Round with GC offset assuming base > offset
    fn Round(&self, base: usize) -> usize {
        base - self.gc_offset
    }

    fn Grow(&mut self, rounds: usize) {
        (0..rounds).for_each(|_| {
            let mut round = Round::new();
            round.resize(self.proc_num + 1, None);
            let mut round_visited = Vec::new();
            round_visited.resize(self.proc_num + 1, false);
            let mut round_ordered = Vec::new();
            round_ordered.resize(self.proc_num + 1, false);

            self.matrix.push_back(round);
            self.visited.push_back(round_visited);
            self.ordered.push_back(round_ordered);
        });
    }

    fn GC(&mut self) {
        let to_gc = self.ordered.len().saturating_sub(GC_REMAIN);
        (0..to_gc).for_each(|_| {
            self.matrix.pop_front();
            self.visited.pop_front();
            self.ordered.pop_front();
        });
        self.gc_offset += to_gc;
    }

    fn Insert(&mut self, v: VertexPtr) {
        let round = self.Round(v.round);
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
        &self.matrix[self.Round(index)]
    }
}
