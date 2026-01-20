use dag_based::rider::DAGRider;
use matrix::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::anykv,
    time::Jiffies,
};

fn main() {
    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<DAGRider>("Validators", 53)
        .LatencyTopology(&[LatencyDescription::WithinPool(
            "Validators",
            Distributions::Normal(Jiffies(50), Jiffies(10)),
        )])
        .TimeBudget(Jiffies(3600_000))
        .NICBandwidth(BandwidthDescription::Unbounded)
        .Seed(123)
        .Build();

    anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));

    sim.Run();

    let ordered = anykv::Get::<(f64, usize)>("avg_latency").1;
    let avg_latency = anykv::Get::<(f64, usize)>("avg_latency").0;
    println!("ordered: {ordered}, avg_latency: {avg_latency}")
}
