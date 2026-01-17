use std::{fs::File, sync::Mutex, time::Instant};

use dag_based::bullshark::Bullshark;
use matrix::{BandwidthType, SimulationBuilder, global::anykv, time::Jiffies};
use rayon::prelude::*;

use std::io::Write;

fn main() {
    let file = File::create("results.csv").unwrap();
    let file = Mutex::new(file);

    (100..=100)
        .step_by(1)
        // .par_bridge()
        .into_iter()
        .for_each(|k_validators| {
            let start = Instant::now();
            // 1 jiffy == 1 real millisecond
            let mut sim = SimulationBuilder::NewDefault()
                .AddPool::<Bullshark>("Validators", k_validators)
                .MaxLatency(Jiffies(400)) // 400 ms of max network latency
                .TimeBudget(Jiffies(240_000)) // Simulating 4 min of real time execution
                .NICBandwidth(BandwidthType::Bounded(10 * 1024 * 1024 * 1024 / (8 * 1000))) // 10Gb/sec NICs
                .Seed(k_validators as u64)
                .Build();

            // (avg_latency, total_vertex)
            anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));
            anykv::Set::<usize>("timeouts-fired", 0);

            sim.Run();
            println!("elapsed: {} millis", start.elapsed().as_millis());
            println!("Simulation done for {k_validators} validators");

            let ordered = anykv::Get::<(f64, usize)>("avg_latency").1;
            let avg_latency = anykv::Get::<(f64, usize)>("avg_latency").0;

            writeln!(
                file.lock().unwrap(),
                "{} {} {}",
                k_validators,
                ordered,
                avg_latency
            )
            .unwrap();
        })
}
