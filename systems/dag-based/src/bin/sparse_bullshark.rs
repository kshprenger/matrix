use std::{fs::File, sync::Mutex};

use dag_based::sparse_bullshark::SparseBullshark;
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
        let file = Mutex::new(File::create(format!("sparse_bullshark_{}.csv", bandwidth)).unwrap());

        let seeds = [4567898765, 33333, 982039, 1, 234567890];
        // 5% to quorum by 5 % step
        let samples = (((k_validators as f64 * 0.05) as usize)
            ..=((k_validators as f64 * 0.66) as usize))
            .step_by((k_validators as f64 * 0.05) as usize);
        let product = samples.flat_map(|x| seeds.iter().map(move |y| (x, y)));

        product.par_bridge().into_par_iter().for_each(|(d, seed)| {
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));
            anykv::Set::<usize>("D", d); // Sample size

            let mut sim = SimulationBuilder::NewDefault()
                .AddPool::<SparseBullshark>("Validators", k_validators)
                .LatencyTopology(&[LatencyDescription::WithinPool(
                    "Validators",
                    Distributions::Normal(Jiffies(50), Jiffies(10)),
                )])
                .TimeBudget(Jiffies(3600_000)) // Simulating hour of real time execution
                .NICBandwidth(BandwidthDescription::Bounded(
                    bandwidth * 1024 * 1024 / (8 * 1000), // bandwidth Mb/sec NICs
                ))
                .Seed(*seed)
                .Build();

            // (avg_latency, total_vertex)
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));

            sim.Run();

            let ordered = anykv::Get::<(f64, usize)>("avg_latency").1;
            let avg_latency = anykv::Get::<(f64, usize)>("avg_latency").0;

            writeln!(file.lock().unwrap(), "{} {} {}", d, ordered, avg_latency).unwrap();
        });
    });
}
