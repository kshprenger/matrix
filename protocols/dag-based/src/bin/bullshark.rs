use dag_based::bullshark::Bullshark;
use simulator::{BandwidthType, Jiffies, SimulationBuilder};
fn main() {
    let mut sim = SimulationBuilder::NewFromFactory(|| Bullshark::New())
        .MaxLatency(Jiffies(0))
        .MaxTime(Jiffies(10))
        .NetworkBandwidth(BandwidthType::Unbounded)
        .ProcessInstances(4)
        .Seed(69)
        .Build();

    let metrics = sim.Run();
}
