use std::{fs::File, sync::Mutex};

use dag_based::bullshark::Bullshark;
use matrix::{BandwidthType, SimulationBuilder, global::anykv, time::Jiffies};
use rayon::prelude::*;

use std::io::Write;

fn main() {
    let file = File::create("results.csv").unwrap();
    let file = Mutex::new(file);

    (4..1000).into_par_iter().for_each(|k_validators| {
        // 1 jiffy == 1 real millisecond
        let sim = SimulationBuilder::NewDefault()
            .AddPool::<Bullshark>("Validators", k_validators)
            .MaxLatency(Jiffies(400)) // 400 ms of max network latency
            .TimeBudget(Jiffies(1200_000)) // Simulating 20 min of real time execution
            .NICBandwidth(BandwidthType::Bounded(1 * 1024 * 1024 * 1024 / (8 * 1000))) // 1Gb/sec NICs
            .Seed(k_validators as u64)
            .Build();

        anykv::Set::<Vec<Jiffies>>("latency", Vec::new());
        anykv::Set::<usize>("timeouts-fired", 0);

        sim.Run();
        println!("Simulation done for {k_validators} validators");

        let ordered = anykv::Get::<Vec<Jiffies>>("latency").len();
        let average_latency = anykv::Get::<Vec<Jiffies>>("latency")
            .iter()
            .map(|&x| x.0 as f64)
            .enumerate()
            .fold(0.0, |acc, (i, x)| acc + (x - acc) / (i + 1) as f64);

        anykv::Clear();
        writeln!(
            file.lock().unwrap(),
            "{} {} {}",
            k_validators,
            ordered,
            average_latency
        )
        .unwrap();
    })
}
