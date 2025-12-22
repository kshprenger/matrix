use crate::{
    Simulation, network_condition::BandwidthType, process::ProcessHandle, random::Seed,
    time::Jiffies,
};

pub struct SimulationBuilder<F, P>
where
    F: Fn() -> P,
    P: ProcessHandle,
{
    seed: Seed,
    max_steps: Jiffies,
    max_network_latency: Jiffies,
    process_count: usize,
    factory: F,
    bandwidth: BandwidthType,
}

impl<F, P> SimulationBuilder<F, P>
where
    F: Fn() -> P,
    P: ProcessHandle,
{
    pub fn NewFromFactory(f: F) -> SimulationBuilder<F, P> {
        SimulationBuilder {
            seed: 69,
            max_steps: Jiffies(1_000_000),
            max_network_latency: Jiffies(10),
            process_count: 5,
            factory: f,
            bandwidth: BandwidthType::Unbounded,
        }
    }

    pub fn Seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    pub fn MaxTime(mut self, max_steps: Jiffies) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn MaxLatency(mut self, max_network_latency: Jiffies) -> Self {
        self.max_network_latency = max_network_latency;
        self
    }

    pub fn ProcessInstances(mut self, count: usize) -> Self {
        self.process_count = count;
        self
    }

    pub fn NetworkBandwidth(mut self, bandwidth: BandwidthType) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    pub fn Build(self) -> Simulation<P> {
        Simulation::New(
            self.seed,
            self.max_steps,
            self.max_network_latency,
            self.bandwidth,
            (1..=self.process_count)
                .map(|id| (id, (self.factory)()))
                .collect(),
        )
    }
}
