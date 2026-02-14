// https://arxiv.org/pdf/2102.08325

use std::{
    collections::BTreeSet,
    rc::{Rc, Weak},
};

use dscale::{global::configuration, *};

use crate::{
    consistent_broadcast::{BCBMessage, ByzantineConsistentBroadcast},
    dag_utils::{RoundBasedDAG, Vertex, VertexMessage, VertexPtr, same_vertex},
};

const CONSTRUCTING_ROUTINE_INTERVAL: Jiffies = Jiffies(500);

#[derive(Default)]
pub struct DAGRider {
    rbcast: ByzantineConsistentBroadcast,
    self_id: ProcessId,
    proc_num: usize,
    dag: RoundBasedDAG,
    round: usize,
    buffer: BTreeSet<VertexPtr>,
    decided_wave: usize,
    leaders_stack: Vec<VertexPtr>,
}

impl ProcessHandle for DAGRider {
    fn start(&mut self) {
        self.self_id = rank();
        self.proc_num = configuration::process_number();
        self.dag.set_round_size(configuration::process_number());
        self.rbcast.start(configuration::process_number());

        schedule_timer_after(CONSTRUCTING_ROUTINE_INTERVAL);

        // Shared genesis vertices
        let genesis_vertex = VertexPtr::new(Vertex {
            round: 0,
            source: self.self_id,
            strong_edges: Vec::new(),
            creation_time: now(),
        });

        self.dag.add_vertex(genesis_vertex.clone());

        self.rbcast
            .reliably_broadcast(VertexMessage::Genesis(genesis_vertex));
    }

    fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(bs_message) = self.rbcast.process(from, message.as_type::<BCBMessage>()) {
            match bs_message.as_type::<VertexMessage>().as_ref() {
                VertexMessage::Genesis(v) => {
                    debug_assert!(v.round == 0);
                    self.dag.add_vertex(v.clone());
                    return;
                }

                VertexMessage::Vertex(v) => {
                    if self.bad_vertex(&v, from) {
                        return;
                    }
                    self.buffer.insert(v.clone());
                }
            }
        }
    }

    fn on_timer(&mut self, _id: TimerId) {
        self.construct();
    }
}

impl DAGRider {
    fn construct(&mut self) {
        let ready_to_be_added = self
            .buffer
            .iter()
            .cloned()
            .filter(|v| v.round <= self.round)
            .filter(|v| {
                v.strong_edges
                    .iter()
                    .map(|weak| weak.upgrade().unwrap())
                    .all(|parent| match self.dag[parent.round][parent.source] {
                        None => false,
                        Some(ref vertex) => same_vertex(&parent, vertex),
                    })
            })
            .collect::<Vec<VertexPtr>>();

        self.buffer.retain(|v| !ready_to_be_added.contains(v));

        ready_to_be_added.into_iter().for_each(|v| {
            self.dag.add_vertex(v.clone());
        });

        self.try_advance_round();
        schedule_timer_after(CONSTRUCTING_ROUTINE_INTERVAL);
    }

    fn try_advance_round(&mut self) {
        if self.quorum_reached_for_round(self.round) {
            if self.round % 4 == 0 && self.round != 0 {
                self.wave_ready(self.round / 4);
            }
            self.round += 1;
            let v = self.create_vertex(self.round);
            self.dag.add_vertex(v.clone());
            self.rbcast.reliably_broadcast(VertexMessage::Vertex(v));
        }
    }
}

// Utils
impl DAGRider {
    fn adversary_threshold(&self) -> usize {
        (self.proc_num - 1) / 3
    }

    fn quorum_size(&self) -> usize {
        2 * self.adversary_threshold() + 1
    }

    fn non_none_vertices_count_for_round(&self, round: usize) -> usize {
        self.dag[round].iter().flatten().count()
    }

    fn quorum_reached_for_round(&self, round: usize) -> bool {
        self.non_none_vertices_count_for_round(round) >= self.quorum_size()
    }

    fn create_vertex(&self, round: usize) -> VertexPtr {
        VertexPtr::new(Vertex {
            round,
            source: self.self_id,
            strong_edges: self.dag[round - 1]
                .iter()
                .flatten()
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

    fn round(&self, w: usize, k: usize) -> usize {
        4 * (w - 1) + k
    }

    fn get_wave_vertex_leader(&self, w: usize) -> Option<VertexPtr> {
        let round = self.round(w, 1);
        let leader = self.get_leader_id(round);
        return self.dag[round][leader].clone();
    }
}

// Consensus logic
impl DAGRider {
    fn wave_ready(&mut self, w: usize) {
        let mut leader = match self.get_wave_vertex_leader(w) {
            None => return,
            Some(leader) => leader,
        };

        let non_none_vertices = self.dag[self.round(w, 4)]
            .iter()
            .filter(|v| v.is_some())
            .map(|v| v.clone().unwrap())
            .collect::<Vec<VertexPtr>>();

        if non_none_vertices
            .into_iter()
            .filter(|v| self.dag.path_exists(&v, &leader))
            .count()
            < self.quorum_size()
        {
            return;
        }

        self.leaders_stack.push(leader.clone());

        for w_ in ((self.decided_wave + 1)..=(w - 1)).rev() {
            let v_ = self.get_wave_vertex_leader(w_);
            if v_.is_some() && self.dag.path_exists(&leader, v_.as_ref().unwrap()) {
                self.leaders_stack.push(v_.clone().unwrap());
                leader = v_.unwrap();
            }
        }
        self.decided_wave = w;
        self.order_vertices()
    }

    fn order_vertices(&mut self) {
        while let Some(leader) = self.leaders_stack.pop() {
            self.dag.order_from(&leader);
        }
    }
}
