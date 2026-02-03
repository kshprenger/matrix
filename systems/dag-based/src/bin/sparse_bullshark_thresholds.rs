use std::{fs::File, sync::Mutex};

use dag_based::sparse_bullshark::SparseBullshark;
use matrix::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::anykv,
    time::Jiffies,
};
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let k_validators = 2000;
    let thresholds = [1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0];

    thresholds.into_iter().for_each(|threshold| {
        let file = Mutex::new(
            File::create(format!("sparse_bullshark_threshold_{}.csv", threshold)).unwrap(),
        );

        let seeds = [1, 2, 3];
        // 5% -> quorum ; by 5% step
        let samples = (((k_validators as f64 * 0.05) as usize)
            ..=((k_validators as f64 * 0.67) as usize))
            .step_by((k_validators as f64 * 0.05) as usize);
        let product = samples.flat_map(|x| seeds.iter().map(move |y| (x, y)));

        product.par_bridge().into_par_iter().for_each(|(d, seed)| {
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));
            anykv::Set::<usize>("D", d); // Sample size
            anykv::Set::<f64>("threshold", threshold); // xf + 1

            let mut sim = SimulationBuilder::NewDefault()
                .AddPool::<SparseBullshark>("Validators", k_validators)
                .LatencyTopology(&[LatencyDescription::WithinPool(
                    "Validators",
                    Distributions::Normal(Jiffies(50), Jiffies(10)),
                )])
                .TimeBudget(Jiffies(36000_000)) // Simulating 10 hours of real time execution
                .NICBandwidth(BandwidthDescription::Bounded(5 * 1024 * 1024 / (8 * 1000)))
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
