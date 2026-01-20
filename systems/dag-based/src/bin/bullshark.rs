use std::{fs::File, sync::Mutex};

use dag_based::bullshark::Bullshark;
use matrix::{
    BandwidthDescription, Distributions, LatencyDescription, SimulationBuilder, global::anykv,
    time::Jiffies,
};

use std::io::Write;

fn main() {
    let file = File::create("results_bullshark.csv").unwrap();
    let file = Mutex::new(file);

    (3000..=3000).into_iter().for_each(|k_validators| {
        // 1 jiffy == 1 real millisecond
        let mut sim = SimulationBuilder::NewDefault()
            .AddPool::<Bullshark>("Validators", k_validators)
            .LatencyTopology(&[LatencyDescription::WithinPool(
                "Validators",
                Distributions::Normal(Jiffies(50), Jiffies(10)),
            )])
            .TimeBudget(Jiffies(3600_000)) // Simulating hour of real time execution
            .NICBandwidth(BandwidthDescription::Bounded(
                5 * 1024 * 1024 / (8 * 1000), // 5Mb/sec NICs
            ))
            .Seed(k_validators as u64)
            .Build();

        // (avg_latency, total_vertex)
        anykv::Set::<(f64, usize)>("avg_latency", (0.0, 0));

        sim.Run();

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
