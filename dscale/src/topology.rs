use std::{collections::HashMap, rc::Rc};

use crate::{ProcessId, random::Distributions};

pub(crate) type LatencyTopology = HashMap<(ProcessId, ProcessId), Distributions>;
pub(crate) type PoolListing = HashMap<String, Vec<ProcessId>>;

pub enum LatencyDescription {
    WithinPool(&'static str, Distributions),
    BetweenPools(&'static str, &'static str, Distributions),
}

pub(crate) struct Topology {
    pool_listing: PoolListing,
    latency_topology: LatencyTopology,
}

impl Topology {
    pub(crate) fn NewShared(
        pool_listing: PoolListing,
        latency_topology: LatencyTopology,
    ) -> Rc<Self> {
        Rc::new(Self {
            pool_listing,
            latency_topology,
        })
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
}
