// https://arxiv.org/pdf/2201.05677
// https://arxiv.org/pdf/2209.05633

use std::{
    collections::BTreeSet,
    rc::{Rc, Weak},
};

use dscale::{global::configuration, *};

use crate::{
    consistent_broadcast::{BCBMessage, ByzantineConsistentBroadcast},
    dag_utils::{RoundBasedDAG, Vertex, VertexMessage, VertexPtr, same_vertex},
};

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
    fn start(&mut self) {
        self.self_id = rank();
        self.proc_num = configuration::process_number();
        self.dag.set_round_size(configuration::process_number());
        self.rbcast.start(configuration::process_number());

        // Shared genesis vertices
        let genesis_vertex = VertexPtr::new(Vertex {
            round: 0,
            source: self.self_id,
            strong_edges: Vec::new(),
            creation_time: now(),
        });

        self.rbcast
            .reliably_broadcast(VertexMessage::Genesis(genesis_vertex));
    }

    // DAG construction: part 1
    fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(bs_message) = self.rbcast.process(from, message.as_type::<BCBMessage>()) {
            match bs_message.as_type::<VertexMessage>().as_ref() {
                VertexMessage::Genesis(v) => {
                    debug_process!("Got genesis");
                    debug_assert!(v.round == 0);
                    self.dag.add_vertex(v.clone());
                    self.try_advance_round();
                    return;
                }

                VertexMessage::Vertex(v) => {
                    debug_process!("Got vertex from: {from}");

                    // Validity check
                    if self.bad_vertex(&v, from) {
                        return;
                    }

                    // Try to drain stalled vertices first (in sorted order)
                    let mut vertices_in_the_buffer =
                        self.buffer.iter().cloned().collect::<Vec<VertexPtr>>();
                    vertices_in_the_buffer.sort_by_key(|v| v.round);
                    vertices_in_the_buffer.into_iter().for_each(|v| {
                        self.try_add_to_dag(v);
                    });

                    // Then try add current received vertex
                    if !self.try_add_to_dag(v.clone()) {
                        self.buffer.insert(v.clone());
                    }

                    if self.round == v.round {
                        if !self.wait {
                            self.try_advance_round();
                            return;
                        }

                        // Note: anchor vertices are on even rounds
                        match self.round % 4 {
                            0 | 2 => {
                                // Wait for steady leader of this round
                                if self.get_anchor(self.round).is_some() {
                                    self.try_advance_round();
                                }
                            }
                            1 | 3 => {
                                // Wait for 2f+1 links for anchor in previous round
                                if self.get_anchor(self.round - 1).is_none() {
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
                                                same_vertex(
                                                    &v,
                                                    &self.get_anchor(self.round - 1).unwrap(),
                                                )
                                            })
                                    })
                                    .count()
                                    >= self.quorum_size()
                                {
                                    self.try_advance_round();
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
    }

    fn on_timer(&mut self, id: TimerId) {
        if id == self.current_timer {
            debug_process!("Timer fired: {id}");
            self.wait = false;
            self.try_advance_round();
        }
    }
}

// Utils
impl Bullshark {
    fn adversary_threshold(&self) -> usize {
        (self.proc_num - 1) / 3
    }

    fn quorum_size(&self) -> usize {
        2 * self.adversary_threshold() + 1
    }

    fn direct_commit_threshold(&self) -> usize {
        self.adversary_threshold() + 1
    }

    fn non_none_vertices_count_for_round(&self, round: usize) -> usize {
        self.dag[round].iter().flatten().count()
    }

    fn quorum_reached_for_round(&self, round: usize) -> bool {
        self.non_none_vertices_count_for_round(round) >= self.quorum_size()
    }

    fn create_vertex(&self, round: usize) -> VertexPtr {
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
            creation_time: now(),
        })
    }

    fn bad_vertex(&self, v: &VertexPtr, from: ProcessId) -> bool {
        v.strong_edges.len() < self.quorum_size() || from != v.source
    }

    fn get_leader_id(&self, round: usize) -> ProcessId {
        return round % self.proc_num + 1;
    }

    fn get_anchor(&self, round: usize) -> Option<VertexPtr> {
        let leader = self.get_leader_id(round);
        self.dag[round][leader].clone()
    }

    fn start_timer(&mut self) {
        self.current_timer = schedule_timer_after(Jiffies(10000));
        debug_process!("New timer scheduled: {}", self.current_timer);
        self.wait = true;
    }
}

// DAG construction: part 2
impl Bullshark {
    fn try_advance_round(&mut self) {
        if self.quorum_reached_for_round(self.round) {
            debug_process!("Advancing to {} round", self.round + 1);
            self.round += 1;
            self.start_timer();
            self.broadcast_vertex(self.round);
        }
    }

    fn broadcast_vertex(&mut self, round: usize) {
        let v = self.create_vertex(round);
        self.try_add_to_dag(v.clone());
        self.rbcast.reliably_broadcast(VertexMessage::Vertex(v));
    }

    fn try_add_to_dag(&mut self, v: VertexPtr) -> bool {
        // Strong edges are not in the DAG yet
        if v.round - 1 > self.dag.current_max_allocated_round() {
            return false;
        }

        let all_strong_edges_in_the_dag = v
            .strong_edges
            .iter()
            .map(|weak| weak.upgrade().unwrap())
            .all(|edge| match self.dag[edge.round][edge.source] {
                None => false,
                Some(ref vertex) => same_vertex(&edge, vertex),
            });

        if !all_strong_edges_in_the_dag {
            return false;
        }

        self.dag.add_vertex(v.clone());

        if self.quorum_reached_for_round(v.round) && v.round > self.round {
            self.round = v.round;
            self.start_timer();
            self.broadcast_vertex(v.round);
        }

        self.buffer.remove(&v);

        if v.source == self.get_leader_id(v.round) {
            self.try_ordering(v);
        }
        return true;
    }
}

// Consensus logic
impl Bullshark {
    fn try_ordering(&mut self, v: VertexPtr) {
        // Note: leaders are on even rounds
        if v.round % 2 == 1 || v.round == 0 {
            return;
        }

        let maybe_anchor = self.get_anchor(v.round - 2);

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
                            .any(|v| same_vertex(&v.upgrade().unwrap(), &anchor))
                    })
                    .count();
                if vote_count >= self.direct_commit_threshold() {
                    self.order_anchors(anchor);
                }
            }
        }
    }

    fn order_anchors(&mut self, v: VertexPtr) {
        let mut anchor = v.clone();
        self.ordered_anchors_stack.push(anchor.clone());
        let mut r = anchor.round.saturating_sub(2); // Ordering can start from second round resulting into negative number here
        while r > self.last_ordered_round {
            let maybe_prev_anchor = self.get_anchor(r);
            match maybe_prev_anchor {
                None => {
                    r = r - 2; // Skip anchor and proceed to the next
                    continue;
                }
                Some(prev_anchor) => {
                    if self.dag.path_exists(&anchor, &prev_anchor) {
                        self.ordered_anchors_stack.push(prev_anchor.clone());
                        anchor = prev_anchor;
                    }
                    r = r - 2;
                }
            }
        }

        self.last_ordered_round = v.round;
        self.order_history();
    }

    fn order_history(&mut self) {
        while let Some(anchor) = self.ordered_anchors_stack.pop() {
            self.dag.order_from(&anchor);
        }
    }
}
