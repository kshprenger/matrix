use std::{fs::File, sync::Mutex};

use dag_based::bullshark::Bullshark;
use dscale::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::anykv,
    time::Jiffies,
};
use rayon::prelude::*;
use std::io::Write;

fn main() {
    let k_validators = 1000;
    let mb_per_sec = [8000, 9000, 10000, 11000];

    mb_per_sec.into_iter().for_each(|bandwidth| {
        let file = Mutex::new(File::create(format!("bullshark_{}.csv", bandwidth)).unwrap());

        let seeds = [4567898765, 33333, 982039];

        seeds.into_par_iter().for_each(|seed| {
            anykv::set::<(f64, usize)>("avg_latency", (0.0, 0));

            let mut sim = SimulationBuilder::default()
                .add_pool::<Bullshark>("Validators", k_validators)
                .latency_topology(&[LatencyDescription::WithinPool(
                    "Validators",
                    Distributions::Normal(Jiffies(50), Jiffies(10)),
                )])
                .time_budget(Jiffies(60_000)) // Simulating 1 min of real time execution
                .nic_bandwidth(BandwidthDescription::Bounded(
                    bandwidth * 1024 * 1024 / (8 * 1000), // bandwidth Mb/sec NICs
                ))
                .seed(seed)
                .build();

            // (avg_latency, total_vertex)
            anykv::set::<(f64, usize)>("avg_latency", (0.0, 0));

            sim.run();

            let ordered = anykv::get::<(f64, usize)>("avg_latency").1;
            let avg_latency = anykv::get::<(f64, usize)>("avg_latency").0;
            let load = anykv::get::<usize>("avg_network_load"); // Bytes per jiffy at single NIC

            writeln!(file.lock().unwrap(), "{} {} {}", ordered, avg_latency, load).unwrap();
        });
    });
}
