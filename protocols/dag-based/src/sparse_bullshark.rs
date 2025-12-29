// https://arxiv.org/pdf/2201.05677
// https://arxiv.org/pdf/2209.05633
// https://arxiv.org/pdf/2506.13998

use std::collections::BTreeSet;

use rand::{SeedableRng, rngs::StdRng};
use simulator::*;

use crate::{
    consistent_broadcast::{BCBMessage, ByzantineConsistentBroadcast},
    dag_utils::{RoundBasedDAG, SameVertex, Vertex, VertexPtr},
};

#[derive(Clone)]
pub enum SparseBullsharkMessage {
    Vertex(VertexPtr),
    Genesis(VertexPtr),
}

impl Message for SparseBullsharkMessage {
    fn VirtualSize(&self) -> usize {
        69
    }
}

pub struct SparseBullshark {
    rbcast: ByzantineConsistentBroadcast,
    proc_num: usize,
    dag: RoundBasedDAG,
    round: usize,
    buffer: BTreeSet<VertexPtr>,
    last_ordered_round: usize,
    ordered_anchors_stack: Vec<VertexPtr>,
    wait: bool,
    current_timer: TimerId,
    sampler: Option<StdRng>,
    D: usize,
}

impl SparseBullshark {
    pub fn New(D: usize) -> Self {
        Self {
            rbcast: ByzantineConsistentBroadcast::New(),
            proc_num: 0,
            dag: RoundBasedDAG::New(),
            round: 0,
            buffer: BTreeSet::new(),
            last_ordered_round: 0,
            ordered_anchors_stack: Vec::new(),
            wait: true,
            current_timer: 0,
            sampler: None,
            D,
        }
    }
}

impl ProcessHandle for SparseBullshark {
    fn Bootstrap(&mut self, configuration: Configuration) {
        self.proc_num = configuration.proc_num;
        self.sampler = Some(StdRng::seed_from_u64(configuration.seed));
        self.dag.SetRoundSize(configuration.proc_num);
        self.rbcast.Bootstrap(configuration);

        // Shared genesis vertices
        let genesis_vertex = VertexPtr::new(Vertex {
            round: 0,
            source: CurrentId(),
            strong_edges: Vec::new(),
            creation_time: time::Now(),
        });

        self.rbcast
            .ReliablyBroadcast(SparseBullsharkMessage::Genesis(genesis_vertex));
    }

    // DAG construction: part 1
    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(bs_message) = self.rbcast.Process(from, message.As::<BCBMessage>()) {
            match bs_message.As::<SparseBullsharkMessage>().as_ref() {
                SparseBullsharkMessage::Genesis(v) => {
                    Debug!("Got genesis");
                    debug_assert!(v.round == 0);
                    self.dag.AddVertex(v.clone());
                    self.TryAdvanceRound();
                    return;
                }

                SparseBullsharkMessage::Vertex(v) => {
                    Debug!("Got vertex from: {from}");
                    debug_assert!(v.strong_edges.len() <= self.D + 2);

                    // Validity check
                    if v.strong_edges.len() < self.QuorumSize() || from != v.source {
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
                        // Note: anchor vertices are on even rounds
                        if !self.wait {
                            self.TryAdvanceRound();
                            return;
                        }

                        match self.round % 4 {
                            0 => {
                                // Wait for first steady leader
                                if self.GetAnchor(self.round).is_some() {
                                    self.TryAdvanceRound();
                                }
                            }
                            2 => {
                                // Wait for second steady leader
                                if self.GetAnchor(self.round).is_some() {
                                    self.TryAdvanceRound();
                                }
                            }
                            1 => {
                                // Wait for 2f+1 links for anchor in previous round
                                if self.GetAnchor(self.round - 1).is_none() {
                                    return;
                                }

                                if self.dag[self.round]
                                    .iter()
                                    .flatten()
                                    .map(|v| {
                                        v.strong_edges
                                            .contains(&self.GetAnchor(self.round - 1).unwrap())
                                    })
                                    .count()
                                    >= self.QuorumSize()
                                {
                                    self.TryAdvanceRound();
                                }
                            }
                            3 => {
                                // Wait 2f+1 links to anchor in previous round
                                if self.GetAnchor(self.round - 1).is_none() {
                                    return;
                                }
                                if self.dag[self.round]
                                    .iter()
                                    .flatten()
                                    .map(|v| {
                                        v.strong_edges
                                            .contains(&self.GetAnchor(self.round - 1).unwrap())
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
            Debug!("Timer fired: {}", id);
            metrics::Modify::<usize>("timeouts-fired", |count| *count += 1);
            self.wait = false;
            self.TryAdvanceRound();
        }
    }
}

// Utils
impl SparseBullshark {
    fn AdversaryThreshold(&self) -> usize {
        (self.proc_num - 1) / 3
    }

    fn QuorumSize(&self) -> usize {
        2 * self.AdversaryThreshold() + 1
    }

    fn DirectCommitThreshold(&self) -> usize {
        2 * self.AdversaryThreshold() + 1
    }

    fn NonNoneVerticesCountForRound(&self, round: usize) -> usize {
        self.dag[round].iter().flatten().count()
    }

    fn QuorumReachedForRound(&self, round: usize) -> bool {
        self.NonNoneVerticesCountForRound(round) >= self.QuorumSize()
    }

    fn SampleCandidates(&mut self, round: usize) -> Vec<VertexPtr> {
        let candidates: Vec<VertexPtr> = self.dag[round].iter().flatten().cloned().collect();

        if candidates.len() <= self.D {
            return candidates;
        }

        use rand::prelude::IndexedRandom;
        let mut random_candidates = candidates
            .choose_multiple(
                self.sampler.as_mut().expect("Sampler not initialized"),
                self.D,
            )
            .cloned()
            .collect::<BTreeSet<VertexPtr>>();

        // Try add myself
        if self.dag[round][CurrentId()].is_some() {
            random_candidates.insert(self.dag[round][CurrentId()].clone().unwrap());
        }

        // Try add anchor
        if self.GetAnchor(round).is_some() {
            random_candidates.insert(self.GetAnchor(round).unwrap());
        }

        debug_assert!(random_candidates.len() >= self.D);
        debug_assert!(random_candidates.len() <= self.D + 2);

        random_candidates.into_iter().collect()
    }

    fn CreateVertex(&mut self, round: usize) -> VertexPtr {
        // Infinite source of client txns
        VertexPtr::new(Vertex {
            round,
            source: CurrentId(),
            strong_edges: self.SampleCandidates(round - 1),
            creation_time: time::Now(),
        })
    }

    fn GetLeaderId(&self, round: usize) -> ProcessId {
        return round % self.proc_num + 1;
    }

    fn GetAnchor(&self, round: usize) -> Option<VertexPtr> {
        let leader = self.GetLeaderId(round);
        self.dag[round][leader].clone()
    }

    fn StartTimer(&mut self) {
        self.current_timer = ScheduleTimerAfter(Jiffies(200));
        Debug!("New timer scheduled: {}", self.current_timer);
        self.wait = true;
    }
}

// DAG construction: part 2
impl SparseBullshark {
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
        self.rbcast
            .ReliablyBroadcast(SparseBullsharkMessage::Vertex(v));
    }

    fn TryAddToDAG(&mut self, v: VertexPtr) -> bool {
        // Strong edges are not in the DAG yet
        if v.round - 1 > self.dag.CurrentMaxAllocatedRound() {
            return false;
        }

        let all_strong_edges_in_the_dag =
            v.strong_edges
                .iter()
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
impl SparseBullshark {
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
                    .filter(|vote| vote.strong_edges.contains(&anchor))
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
        while !self.ordered_anchors_stack.is_empty() {
            let anchor = self
                .ordered_anchors_stack
                .pop()
                .expect("Should not be empty");

            self.dag.OrderFrom(&anchor);
        }
    }
}
