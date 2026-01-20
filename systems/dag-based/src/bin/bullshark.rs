use std::{fs::File, sync::Mutex};

use dag_based::bullshark::Bullshark;
use matrix::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::anykv,
    time::Jiffies,
};
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let k_validators = 1000;
    let mb_per_sec = [5, 10, 20];

    mb_per_sec.into_iter().for_each(|bandwidth| {
        let file = Mutex::new(File::create(format!("bullshark_{}.csv", bandwidth)).unwrap());

        let seeds = [4567898765, 33333, 982039, 1, 234567890];

        seeds.into_par_iter().for_each(|seed| {
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));

            let mut sim = SimulationBuilder::NewDefault()
                .AddPool::<Bullshark>("Validators", k_validators)
                .LatencyTopology(&[LatencyDescription::WithinPool(
                    "Validators",
                    Distributions::Normal(Jiffies(50), Jiffies(10)),
                )])
                .TimeBudget(Jiffies(3600_000)) // Simulating hour of real time execution
                .NICBandwidth(BandwidthDescription::Bounded(
                    bandwidth * 1024 * 1024 / (8 * 1000), // bandwidth Mb/sec NICs
                ))
                .Seed(seed)
                .Build();

            // (avg_latency, total_vertex)
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));

            sim.Run();

            let ordered = anykv::Get::<(f64, usize)>("avg_latency").1;
            let avg_latency = anykv::Get::<(f64, usize)>("avg_latency").0;

            writeln!(file.lock().unwrap(), "{} {}", ordered, avg_latency).unwrap();
        });
    });
}
