use std::{fs::File, sync::Mutex};

use dag_based::bullshark::Bullshark;
use matrix::{BandwidthType, SimulationBuilder, global::anykv, time::Jiffies};
use rayon::prelude::*;

use std::io::Write;

fn main() {
    let file = File::create("results.csv").unwrap();
    let file = Mutex::new(file);

    (4..3000).into_par_iter().for_each(|k_validators| {
        // 1 jiffy == 1 real millisecond
        let sim = SimulationBuilder::NewDefault()
            .AddPool::<Bullshark>("Validators", k_validators)
            .MaxLatency(Jiffies(400)) // 400 ms of max network latency
            .TimeBudget(Jiffies(1200_000)) // Simulating 20 min of real time execution
            .NICBandwidth(BandwidthType::Bounded(10 * 1024 * 1024 * 1024 / (8 * 1000))) // 10Gb/sec NICs
            .Seed(k_validators as u64)
            .Build();

        // (avg_latency, total_vertex)
        anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));
        anykv::Set::<usize>("timeouts-fired", 0);

        sim.Run();
        println!("Simulation done for {k_validators} validators");

        let ordered = anykv::Get::<(f64, usize)>("avg_latency").1;
        let avg_latency = anykv::Get::<(f64, usize)>("avg_latency").0;

        anykv::Clear();
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
