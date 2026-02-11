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
    let mb_per_sec = [3000, 4000, 5000, 6000, 7000];

    mb_per_sec.into_iter().for_each(|bandwidth| {
        let file = Mutex::new(File::create(format!("sparse_bullshark_{}.csv", bandwidth)).unwrap());

        let seeds = [4567898765, 33333, 982039];
        // 5% to quorum by 5 % step
        let samples = (((k_validators as f64 * 0.05) as usize)
            ..=((k_validators as f64 * 0.66) as usize))
            .step_by((k_validators as f64 * 0.05) as usize);
        let product = samples.flat_map(|x| seeds.iter().map(move |y| (x, y)));

        product.par_bridge().into_par_iter().for_each(|(d, seed)| {
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));
            anykv::Set::<(f64, usize)>("avg_virtual_size", (0.0, 0));
            anykv::Set::<usize>("D", d); // Sample size

            let mut sim = SimulationBuilder::NewDefault()
                .AddPool::<SparseBullshark>("Validators", k_validators)
                .LatencyTopology(&[LatencyDescription::WithinPool(
                    "Validators",
                    Distributions::Normal(Jiffies(50), Jiffies(10)),
                )])
                .TimeBudget(Jiffies(60_000)) // Simulating 1 min of real time execution
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
            let load = anykv::Get::<usize>("avg_network_load"); // Bytes per jiffy at single NIC
            let avg_virtual_size_of_message = anykv::Get::<(f64, usize)>("avg_virtual_size");

            writeln!(
                file.lock().unwrap(),
                "{} {} {} {} {}",
                d,
                ordered,
                avg_latency,
                load,
                avg_virtual_size_of_message.0,
            )
            .unwrap();
        });
    });
}
