use std::collections::HashMap;

use crate::{
    ProcessHandle, ProcessId, Simulation,
    network::BandwidthType,
    process::UniqueProcessHandle,
    random::Seed,
    time::{Jiffies, clock},
    tso,
};

// There are a lot of Rc small allocations, so we optimize this too using different allocator
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn InitLogger() {
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
    max_network_latency: Jiffies,
    proc_id: usize,
    pools: HashMap<String, Vec<(ProcessId, UniqueProcessHandle)>>,
    bandwidth: BandwidthType,
}

impl SimulationBuilder {
    pub fn NewDefault() -> SimulationBuilder {
        SimulationBuilder {
            seed: 69,
            time_budget: Jiffies(1_000_000),
            max_network_latency: Jiffies(10),
            proc_id: 1,
            pools: HashMap::new(),
            bandwidth: BandwidthType::Unbounded,
        }
    }

    pub fn AddPool<P: ProcessHandle + 'static>(
        mut self,
        name: &str,
        size: usize,
        factory: impl Fn() -> P,
    ) -> SimulationBuilder {
        let pool = self.pools.entry(name.to_string()).or_default();
        for _ in 0..size {
            let id = self.proc_id;
            self.proc_id += 1;
            pool.push((id, Box::new(factory())));
        }
        self
    }

    pub fn Seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    pub fn TimeBudget(mut self, time_budget: Jiffies) -> Self {
        self.time_budget = time_budget;
        self
    }

    pub fn MaxLatency(mut self, max_network_latency: Jiffies) -> Self {
        self.max_network_latency = max_network_latency;
        self
    }

    pub fn NICBandwidth(mut self, bandwidth: BandwidthType) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    pub fn Build(self) -> Simulation {
        InitLogger();

        // thread_locals may be reused in other simulations, so we need to reset them
        tso::Reset();
        clock::Reset();

        Simulation::New(
            self.seed,
            self.time_budget,
            self.max_network_latency,
            self.bandwidth,
            self.pools,
        )
    }
}
