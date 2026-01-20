// https://arxiv.org/pdf/2102.08325

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
pub enum DAGRiderMessage {
    Vertex(VertexPtr),
    Genesis(VertexPtr),
}

const CONSTRUCTING_ROUTINE_INTERVAL: Jiffies = Jiffies(500);

impl Message for DAGRiderMessage {
    fn VirtualSize(&self) -> usize {
        0
    }
}

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
    fn Start(&mut self) {
        self.self_id = CurrentId();
        self.proc_num = configuration::ProcessNumber();
        self.dag.SetRoundSize(configuration::ProcessNumber());
        self.rbcast.Start(configuration::ProcessNumber());

        ScheduleTimerAfter(CONSTRUCTING_ROUTINE_INTERVAL);

        // Shared genesis vertices
        let genesis_vertex = VertexPtr::new(Vertex {
            round: 0,
            source: self.self_id,
            strong_edges: Vec::new(),
            creation_time: Now(),
        });

        self.dag.AddVertex(genesis_vertex.clone());

        self.rbcast
            .ReliablyBroadcast(DAGRiderMessage::Genesis(genesis_vertex));
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(bs_message) = self.rbcast.Process(from, message.As::<BCBMessage>()) {
            match bs_message.As::<DAGRiderMessage>().as_ref() {
                DAGRiderMessage::Genesis(v) => {
                    debug_assert!(v.round == 0);
                    self.dag.AddVertex(v.clone());
                    return;
                }

                DAGRiderMessage::Vertex(v) => {
                    if self.BadVertex(&v, from) {
                        return;
                    }
                    self.buffer.insert(v.clone());
                }
            }
        }
    }

    fn OnTimer(&mut self, _id: TimerId) {
        self.Construct();
    }
}

impl DAGRider {
    fn Construct(&mut self) {
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
                        Some(ref vertex) => SameVertex(&parent, vertex),
                    })
            })
            .collect::<Vec<VertexPtr>>();

        self.buffer.retain(|v| !ready_to_be_added.contains(v));

        ready_to_be_added.into_iter().for_each(|v| {
            self.dag.AddVertex(v.clone());
        });

        self.TryAdvanceRound();
        ScheduleTimerAfter(CONSTRUCTING_ROUTINE_INTERVAL);
    }

    fn TryAdvanceRound(&mut self) {
        if self.QuorumReachedForRound(self.round) {
            if self.round % 4 == 0 && self.round != 0 {
                self.WaveReady(self.round / 4);
            }
            self.round += 1;
            let v = self.CreateVertex(self.round);
            self.dag.AddVertex(v.clone());
            self.rbcast.ReliablyBroadcast(DAGRiderMessage::Vertex(v));
        }
    }
}

// Utils
impl DAGRider {
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
        VertexPtr::new(Vertex {
            round,
            source: self.self_id,
            strong_edges: self.dag[round - 1]
                .iter()
                .flatten()
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

    fn Round(&self, w: usize, k: usize) -> usize {
        4 * (w - 1) + k
    }

    fn GetWaveVertexLeader(&self, w: usize) -> Option<VertexPtr> {
        let round = self.Round(w, 1);
        let leader = self.GetLeaderId(round);
        return self.dag[round][leader].clone();
    }
}

// Consensus logic
impl DAGRider {
    fn WaveReady(&mut self, w: usize) {
        let mut leader = match self.GetWaveVertexLeader(w) {
            None => return,
            Some(leader) => leader,
        };

        let non_none_vertices = self.dag[self.Round(w, 4)]
            .iter()
            .filter(|v| v.is_some())
            .map(|v| v.clone().unwrap())
            .collect::<Vec<VertexPtr>>();

        if non_none_vertices
            .into_iter()
            .filter(|v| self.dag.PathExists(&v, &leader))
            .count()
            < self.QuorumSize()
        {
            return;
        }

        self.leaders_stack.push(leader.clone());

        for w_ in ((self.decided_wave + 1)..=(w - 1)).rev() {
            let v_ = self.GetWaveVertexLeader(w_);
            if v_.is_some() && self.dag.PathExists(&leader, v_.as_ref().unwrap()) {
                self.leaders_stack.push(v_.clone().unwrap());
                leader = v_.unwrap();
            }
        }
        self.decided_wave = w;
        self.OrderVertices()
    }

    fn OrderVertices(&mut self) {
        while let Some(leader) = self.leaders_stack.pop() {
            self.dag.OrderFrom(&leader);
        }
    }
}
