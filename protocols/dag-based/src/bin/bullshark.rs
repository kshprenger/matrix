use dag_based::bullshark::Bullshark;
use simulator::{BandwidthType, Jiffies, SimulationBuilder};
fn main() {
    let mut sim = SimulationBuilder::NewFromFactory(|| Bullshark::New())
        .MaxLatency(Jiffies(500))
        .MaxTime(Jiffies(100000))
        .NetworkBandwidth(BandwidthType::Unbounded)
        .ProcessInstances(60)
        .Seed(234565432345)
        .Build();

    let _ = sim.Run();
}
