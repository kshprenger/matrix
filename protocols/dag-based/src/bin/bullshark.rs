use dag_based::bullshark::Bullshark;
use simulator::{BandwidthType, SimulationBuilder, metrics, time::Jiffies};
fn main() {
    metrics::Clear();
    metrics::Set::<Vec<Jiffies>>("latency", Vec::new());

    SimulationBuilder::NewFromFactory(|| Bullshark::New())
        .MaxLatency(Jiffies(500))
        .MaxTime(Jiffies(10000_000))
        .NetworkBandwidth(BandwidthType::Bounded(100))
        .ProcessInstances(60)
        .Seed(234565432345)
        .Build()
        .Run();
    println!("{:?}", metrics::Get::<Vec<Jiffies>>("latency").unwrap())
}
