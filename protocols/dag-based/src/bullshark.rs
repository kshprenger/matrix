// https://arxiv.org/pdf/2201.05677
// https://arxiv.org/pdf/2209.05633

use std::collections::BTreeSet;

use simulator::*;

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
        69
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
}

impl Bullshark {
    pub fn New() -> Self {
        Self {
            rbcast: ByzantineConsistentBroadcast::New(),
            self_id: 0,
            proc_num: 0,
            dag: RoundBasedDAG::New(),
            round: 0,
            buffer: BTreeSet::new(),
            last_ordered_round: 0,
            ordered_anchors_stack: Vec::new(),
        }
    }
}

impl ProcessHandle for Bullshark {
    fn Bootstrap(&mut self, configuration: Configuration) {
        self.self_id = configuration.assigned_id;
        self.proc_num = configuration.proc_num;
        self.dag.SetRoundSize(configuration.proc_num);
        self.rbcast.Bootstrap(configuration);

        // Shared genesis vertices
        let genesis_vertex = VertexPtr::new(Vertex {
            round: 0,
            source: self.self_id,
            strong_edges: Vec::new(),
        });

        self.rbcast
            .ReliablyBroadcast(BullsharkMessage::Genesis(genesis_vertex));
    }

    // DAG construction: part 1
    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(bs_message) = self.rbcast.Process(from, message.As::<BCBMessage>()) {
            match bs_message.As::<BullsharkMessage>().as_ref() {
                BullsharkMessage::Genesis(v) => {
                    debug_assert!(v.round == 0);
                    self.dag.AddVertex(v.clone());
                    self.TryAdvanceRound();
                    return;
                }

                BullsharkMessage::Vertex(v) => {
                    if v.strong_edges.len() < self.QuorumSize() || from != v.source {
                        return;
                    }

                    let vertices_in_the_buffer =
                        self.buffer.iter().cloned().collect::<Vec<VertexPtr>>();
                    vertices_in_the_buffer.into_iter().for_each(|v| {
                        self.TryAddToDAG(v);
                    });

                    if !self.TryAddToDAG(v.clone()) {
                        self.buffer.insert(v.clone());
                    }

                    self.TryAdvanceRound();
                }
            }
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
                .collect::<Vec<VertexPtr>>(),
        })
    }

    #[allow(unused)]
    fn GetFirstPredefinedLeader(&self, w: usize) -> ProcessId {
        let round = 4 * w - 3;
        return self.GetLeaderId(round);
    }

    #[allow(unused)]
    fn GetSecondPredefinedLeader(&self, w: usize) -> ProcessId {
        let round = 4 * w - 1;
        return self.GetLeaderId(round);
    }

    fn GetLeaderId(&self, round: usize) -> ProcessId {
        return round % self.proc_num;
    }

    fn GetAnchor(&self, round: usize) -> Option<VertexPtr> {
        let leader = self.GetLeaderId(round);
        self.dag[round][leader].clone()
    }
}

// DAG construction: part 2
impl Bullshark {
    fn TryAdvanceRound(&mut self) {
        if self.QuorumReachedForRound(self.round) {
            self.round += 1;
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
        // Leaders are on even rounds
        if v.round % 2 == 1 && v.round != 0 {
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
                if vote_count >= self.AdversaryThreshold() + 1 {
                    self.OrderAnchors(anchor);
                }
            }
        }
    }

    fn OrderAnchors(&mut self, v: VertexPtr) {
        let mut anchor = v.clone();
        self.ordered_anchors_stack.push(anchor.clone());
        let mut r = anchor.round - 2;
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
