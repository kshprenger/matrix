use std::{fs::File, sync::Mutex};

use dag_based::sparse_bullshark::SparseBullshark;
use matrix::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::anykv,
    time::Jiffies,
};
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let file = File::create("results_sparse_bullshark.csv").unwrap();
    let file = Mutex::new(file);

    (150..=2000)
        .step_by(100)
        .par_bridge()
        .into_par_iter()
        .for_each(|d| {
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));
            anykv::Set::<usize>("D", d); // Sample size

            let mut sim = SimulationBuilder::NewDefault()
                .AddPool::<SparseBullshark>("Validators", 3000)
                .LatencyTopology(&[LatencyDescription::WithinPool(
                    "Validators",
                    Distributions::Normal(Jiffies(50), Jiffies(10)),
                )])
                .TimeBudget(Jiffies(3600_000)) // Simulating 40 min of real time execution
                .NICBandwidth(BandwidthDescription::Bounded(
                    5 * 1024 * 1024 / (8 * 1000), // 5Mb /sec NICs
                ))
                .Seed(d as u64)
                .Build();

            // (avg_latency, total_vertex)
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));

            sim.Run();

            let ordered = anykv::Get::<(f64, usize)>("avg_latency").1;
            let avg_latency = anykv::Get::<(f64, usize)>("avg_latency").0;

            writeln!(file.lock().unwrap(), "{} {} {}", d, ordered, avg_latency).unwrap();
        });
}
