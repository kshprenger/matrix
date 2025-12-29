use dag_based::sparse_bullshark::SparseBullshark;
use simulator::{BandwidthType, SimulationBuilder, metrics, time::Jiffies};

fn main() {
    metrics::Set::<Vec<Jiffies>>("latency", Vec::new());
    metrics::Set::<usize>("timeouts-fired", 0);

    SimulationBuilder::NewFromFactory(|| Box::new(SparseBullshark::New(200)))
        .MaxLatency(Jiffies(0))
        .MaxTime(Jiffies(1000))
        .NICBandwidth(BandwidthType::Unbounded)
        .ProcessInstances(1000)
        .Seed(234565432345)
        .Build()
        .Run();

    println!(
        "Vertices ordered: {}",
        metrics::Get::<Vec<Jiffies>>("latency").unwrap().len()
    );
    println!(
        "Latency distribution: {:?}",
        metrics::Get::<Vec<Jiffies>>("latency").unwrap()
    );
    println!(
        "Timeouts fired: {}",
        metrics::Get::<usize>("timeouts-fired").unwrap()
    );
}
