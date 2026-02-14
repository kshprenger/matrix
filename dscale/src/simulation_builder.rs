use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

use crate::{
    ProcessHandle, ProcessId, Simulation,
    network::BandwidthDescription,
    process::UniqueProcessHandle,
    random::Seed,
    time::Jiffies,
    topology::{LatencyDescription, LatencyTopology},
};

fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let module_path = record.module_path().unwrap_or("unknown");
            let crate_name = module_path.split("::").next().unwrap_or(module_path);
            use std::io::Write;
            writeln!(buf, "[{}] {}", crate_name, record.args())
        })
        .try_init();
}

pub struct SimulationBuilder {
    seed: Seed,
    time_budget: Jiffies,
    proc_id: usize,
    pools: HashMap<String, Vec<(ProcessId, UniqueProcessHandle)>>,
    latency_topology: LatencyTopology,
    bandwidth: BandwidthDescription,
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        SimulationBuilder {
            seed: 69,
            time_budget: Jiffies(1_000_000),
            proc_id: 1,
            pools: HashMap::new(),
            bandwidth: BandwidthDescription::Unbounded,
            latency_topology: HashMap::new(),
        }
    }
}

impl SimulationBuilder {
    pub fn add_pool<P: ProcessHandle + Default + 'static>(
        mut self,
        name: &str,
        size: usize,
    ) -> SimulationBuilder {
        let pool = self.pools.entry(name.to_string()).or_default();
        for _ in 0..size {
            let id = self.proc_id;
            self.proc_id += 1;
            pool.push((id, Box::new(P::default())));
        }
        self
    }

    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    pub fn time_budget(mut self, time_budget: Jiffies) -> Self {
        self.time_budget = time_budget;
        self
    }

    // Should be called only after all add_pool calls
    pub fn latency_topology(mut self, descriptions: &[LatencyDescription]) -> Self {
        descriptions.iter().for_each(|d| {
            let (from, to, distr) = match d {
                LatencyDescription::WithinPool(name, distr) => (*name, *name, distr),
                LatencyDescription::BetweenPools(pool_from, pool_to, distr) => {
                    (*pool_from, *pool_to, distr)
                }
            };

            let from_vec: Vec<ProcessId> = self
                .pools
                .get(from)
                .expect("No pool found")
                .iter()
                .map(|(id, _)| *id)
                .collect();

            let to_vec: Vec<ProcessId> = self
                .pools
                .get(to)
                .expect("No pool found")
                .iter()
                .map(|(id, _)| *id)
                .collect();

            let cartesian_product = from_vec
                .iter()
                .flat_map(|x| to_vec.iter().map(move |y| (*x, *y)));

            let cartesian_product_backwards = from_vec
                .iter()
                .flat_map(|x| to_vec.iter().map(move |y| (*y, *x)));

            cartesian_product.for_each(|key| {
                self.latency_topology.insert(key, distr.clone());
            });

            cartesian_product_backwards.for_each(|key| {
                self.latency_topology.insert(key, distr.clone());
            });
        });
        self
    }

    pub fn nic_bandwidth(mut self, bandwidth: BandwidthDescription) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    pub fn build(self) -> Simulation {
        init_logger();

        let mut pool_listing = HashMap::new();
        let mut procs = BTreeMap::new();

        for (name, pool) in self.pools {
            let mut ids = Vec::new();
            for (id, handle) in pool {
                ids.push(id);
                procs.insert(id, RefCell::new(handle));
            }
            pool_listing.insert(name, ids);
        }

        Simulation::new(
            self.seed,
            self.time_budget,
            self.bandwidth,
            self.latency_topology,
            pool_listing,
            procs,
        )
    }
}
