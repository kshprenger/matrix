// https://arxiv.org/pdf/2201.05677
// https://arxiv.org/pdf/2209.05633

use std::{
    collections::BTreeSet,
    rc::{Rc, Weak},
};

use matrix::{global::configuration, *};

use crate::{
    consistent_broadcast::{BCBMessage, ByzantineConsistentBroadcast},
    dag_utils::{RoundBasedDAG, SameVertex, Vertex, VertexPtr},
};

#[derive(Clone)]
pub enum BullsharkMessage {
    Vertex(VertexPtr),
    Genesis(VertexPtr),
}

impl Message for BullsharkMessage {
    fn VirtualSize(&self) -> usize {
        // Round, ProcessId
        4 + 4
            + match self {
                BullsharkMessage::Genesis(v) => v.strong_edges.len() * 32, // sha256 block pointers
                BullsharkMessage::Vertex(v) => v.strong_edges.len() * 32,  // sha256 block pointers
            }
    }
}

pub struct Bullshark {
    rbcast: ByzantineConsistentBroadcast,
    self_id: ProcessId,
    proc_num: usize,
    dag: RoundBasedDAG,
    round: usize,
    buffer: BTreeSet<VertexPtr>,
    last_ordered_round: usize,
    ordered_anchors_stack: Vec<VertexPtr>,
    wait: bool,
    current_timer: TimerId,
}

impl Default for Bullshark {
    fn default() -> Self {
        Self {
            rbcast: ByzantineConsistentBroadcast::default(),
            self_id: 0,
            proc_num: 0,
            dag: RoundBasedDAG::default(),
            round: 0,
            buffer: BTreeSet::new(),
            last_ordered_round: 0,
            ordered_anchors_stack: Vec::new(),
            wait: true,
            current_timer: 0,
        }
    }
}

impl ProcessHandle for Bullshark {
    fn Start(&mut self) {
        self.self_id = CurrentId();
        self.proc_num = configuration::ProcessNumber();
        self.dag.SetRoundSize(configuration::ProcessNumber());
        self.rbcast.Start(configuration::ProcessNumber());

        // Shared genesis vertices
        let genesis_vertex = VertexPtr::new(Vertex {
            round: 0,
            source: self.self_id,
            strong_edges: Vec::new(),
            creation_time: Now(),
        });

        self.rbcast
            .ReliablyBroadcast(BullsharkMessage::Genesis(genesis_vertex));
    }

    // DAG construction: part 1
    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(bs_message) = self.rbcast.Process(from, message.As::<BCBMessage>()) {
            match bs_message.As::<BullsharkMessage>().as_ref() {
                BullsharkMessage::Genesis(v) => {
                    Debug!("Got genesis");
                    debug_assert!(v.round == 0);
                    self.dag.AddVertex(v.clone());
                    self.TryAdvanceRound();
                    return;
                }

                BullsharkMessage::Vertex(v) => {
                    Debug!("Got vertex from: {from}");

                    // Validity check
                    if self.BadVertex(&v, from) {
                        return;
                    }

                    // Try to drain stalled vertices first
                    let vertices_in_the_buffer =
                        self.buffer.iter().cloned().collect::<Vec<VertexPtr>>();
                    vertices_in_the_buffer.into_iter().for_each(|v| {
                        self.TryAddToDAG(v);
                    });

                    // Then try add current received vertex
                    if !self.TryAddToDAG(v.clone()) {
                        self.buffer.insert(v.clone());
                    }

                    if self.round == v.round {
                        if !self.wait {
                            self.TryAdvanceRound();
                            return;
                        }

                        // Note: anchor vertices are on even rounds
                        match self.round % 4 {
                            0 | 2 => {
                                // Wait for steady leader of this round
                                if self.GetAnchor(self.round).is_some() {
                                    self.TryAdvanceRound();
                                }
                            }
                            1 | 3 => {
                                // Wait for 2f+1 links for anchor in previous round
                                if self.GetAnchor(self.round - 1).is_none() {
                                    return;
                                }

                                if self.dag[self.round]
                                    .iter()
                                    .flatten()
                                    .map(|v| {
                                        v.strong_edges
                                            .iter()
                                            .map(|weak| weak.upgrade().unwrap())
                                            .any(|v| {
                                                SameVertex(
                                                    &v,
                                                    &self.GetAnchor(self.round - 1).unwrap(),
                                                )
                                            })
                                    })
                                    .count()
                                    >= self.QuorumSize()
                                {
                                    self.TryAdvanceRound();
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
    }

    fn OnTimer(&mut self, id: TimerId) {
        if id == self.current_timer {
            Debug!("Timer fired: {id}");
            self.wait = false;
            self.TryAdvanceRound();
        }
    }
}

// Utils
impl Bullshark {
    fn AdversaryThreshold(&self) -> usize {
        (self.proc_num - 1) / 3
    }

    fn QuorumSize(&self) -> usize {
        2 * self.AdversaryThreshold() + 1
    }

    fn DirectCommitThreshold(&self) -> usize {
        self.AdversaryThreshold() + 1
    }

    fn NonNoneVerticesCountForRound(&self, round: usize) -> usize {
        self.dag[round].iter().flatten().count()
    }

    fn QuorumReachedForRound(&self, round: usize) -> bool {
        self.NonNoneVerticesCountForRound(round) >= self.QuorumSize()
    }

    fn CreateVertex(&self, round: usize) -> VertexPtr {
        // Infinite source of client txns
        VertexPtr::new(Vertex {
            round,
            source: self.self_id,
            strong_edges: self.dag[round - 1]
                .iter()
                .flatten() // Remove option
                .cloned()
                .map(|strong| Rc::downgrade(&strong))
                .collect::<Vec<Weak<Vertex>>>(),
            creation_time: Now(),
        })
    }

    fn BadVertex(&self, v: &VertexPtr, from: ProcessId) -> bool {
        v.strong_edges.len() < self.QuorumSize() || from != v.source
    }

    fn GetLeaderId(&self, round: usize) -> ProcessId {
        return round % self.proc_num + 1;
    }

    fn GetAnchor(&self, round: usize) -> Option<VertexPtr> {
        let leader = self.GetLeaderId(round);
        self.dag[round][leader].clone()
    }

    fn StartTimer(&mut self) {
        self.current_timer = ScheduleTimerAfter(Jiffies(2000));
        Debug!("New timer scheduled: {}", self.current_timer);
        self.wait = true;
    }
}

// DAG construction: part 2
impl Bullshark {
    fn TryAdvanceRound(&mut self) {
        if self.QuorumReachedForRound(self.round) {
            Debug!("Advancing to {} round", self.round + 1);
            self.round += 1;
            self.StartTimer();
            self.BroadcastVertex(self.round);
        }
    }

    fn BroadcastVertex(&mut self, round: usize) {
        let v = self.CreateVertex(round);
        self.TryAddToDAG(v.clone());
        self.rbcast.ReliablyBroadcast(BullsharkMessage::Vertex(v));
    }

    fn TryAddToDAG(&mut self, v: VertexPtr) -> bool {
        // Strong edges are not in the DAG yet
        if v.round - 1 > self.dag.CurrentMaxAllocatedRound() {
            return false;
        }

        let all_strong_edges_in_the_dag = v
            .strong_edges
            .iter()
            .map(|weak| weak.upgrade().unwrap())
            .all(|edge| match self.dag[edge.round][edge.source] {
                None => false,
                Some(ref vertex) => SameVertex(&edge, vertex),
            });

        if !all_strong_edges_in_the_dag {
            return false;
        }

        self.dag.AddVertex(v.clone());

        if self.QuorumReachedForRound(v.round) && v.round > self.round {
            self.round = v.round;
            self.StartTimer();
            self.BroadcastVertex(v.round);
        }

        self.buffer.remove(&v);

        if v.source == self.GetLeaderId(v.round) {
            self.TryOrdering(v);
        }
        return true;
    }
}

// Consensus logic
impl Bullshark {
    fn TryOrdering(&mut self, v: VertexPtr) {
        // Note: leaders are on even rounds
        if v.round % 2 == 1 || v.round == 0 {
            return;
        }

        let maybe_anchor = self.GetAnchor(v.round - 2);

        match maybe_anchor {
            None => return,
            Some(anchor) => {
                let vote_count = v
                    .strong_edges
                    .iter()
                    .map(|weak| weak.upgrade().unwrap())
                    .filter(|vote| {
                        vote.strong_edges
                            .iter()
                            .any(|v| SameVertex(&v.upgrade().unwrap(), &anchor))
                    })
                    .count();
                if vote_count >= self.DirectCommitThreshold() {
                    self.OrderAnchors(anchor);
                }
            }
        }
    }

    fn OrderAnchors(&mut self, v: VertexPtr) {
        let mut anchor = v.clone();
        self.ordered_anchors_stack.push(anchor.clone());
        let mut r = anchor.round.saturating_sub(2); // Ordering can start from second round resulting into negative number here
        while r > self.last_ordered_round {
            let maybe_prev_anchor = self.GetAnchor(r);
            match maybe_prev_anchor {
                None => {
                    r = r - 2; // Skip anchor and proceed to the next
                    continue;
                }
                Some(prev_anchor) => {
                    if self.dag.PathExists(&anchor, &prev_anchor) {
                        self.ordered_anchors_stack.push(prev_anchor.clone());
                        anchor = prev_anchor;
                    }
                    r = r - 2;
                }
            }
        }

        self.last_ordered_round = v.round;
        self.OrderHistory();
    }

    fn OrderHistory(&mut self) {
        while let Some(anchor) = self.ordered_anchors_stack.pop() {
            self.dag.OrderFrom(&anchor);
        }
    }
}
