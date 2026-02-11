use std::{
    cell::RefMut,
    collections::{BTreeMap, HashMap, btree_map::Keys},
    rc::Rc,
};

use crate::{
    ProcessId,
    communication::DScaleMessage,
    global::SetProcess,
    process::{MutableProcessHandle, UniqueProcessHandle},
    random::Distributions,
};

pub(crate) type LatencyTopology = HashMap<(ProcessId, ProcessId), Distributions>;
pub(crate) type PoolListing = HashMap<String, Vec<ProcessId>>;
pub(crate) type HandlerMap = BTreeMap<ProcessId, MutableProcessHandle>; // btree for deterministic iterators

pub enum LatencyDescription {
    WithinPool(&'static str, Distributions),
    BetweenPools(&'static str, &'static str, Distributions),
}

pub(crate) struct Topology {
    procs: HandlerMap,
    pool_listing: PoolListing,
    latency_topology: LatencyTopology,
}

impl Topology {
    pub(crate) fn NewShared(
        procs: HandlerMap,
        pool_listing: PoolListing,
        latency_topology: LatencyTopology,
    ) -> Rc<Self> {
        Rc::new(Self {
            procs,
            pool_listing,
            latency_topology,
        })
    }

    pub(crate) fn Deliver(&self, from: ProcessId, to: ProcessId, m: DScaleMessage) {
        let mut handle = self.procs.get(&to).expect("Invalid ProcessId").borrow_mut();
        SetProcess(to);
        match m {
            DScaleMessage::NetworkMessage(ptr) => handle.OnMessage(from, ptr),
            DScaleMessage::Timer(id) => handle.OnTimer(id),
        }
    }

    pub(crate) fn GetDistribution(&self, from: ProcessId, to: ProcessId) -> Distributions {
        self.latency_topology
            .get(&(from, to))
            .copied()
            .expect("No distr found")
    }

    pub(crate) fn ListPool(&self, pool_name: &str) -> &[usize] {
        self.pool_listing.get(pool_name).expect("Invalid pool name")
    }

    // Note: deterministic
    pub(crate) fn IterMut(
        &self,
    ) -> impl Iterator<Item = (&ProcessId, RefMut<'_, UniqueProcessHandle>)> {
        self.procs
            .iter()
            .map(|(id, handle)| (id, handle.borrow_mut()))
    }

    pub(crate) fn Keys(&self) -> Keys<'_, ProcessId, MutableProcessHandle> {
        self.procs.keys()
    }

    pub(crate) fn Size(&self) -> usize {
        self.procs.len()
    }
}
