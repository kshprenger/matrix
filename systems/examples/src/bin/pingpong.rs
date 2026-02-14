use std::time::Instant;

use dscale::{global::anykv, *};
use examples::pingpong::PingPongProcess;

fn main() {
    let mut sim = SimulationBuilder::default()
        .add_pool::<PingPongProcess>("ExamplePool", 2)
        .nic_bandwidth(BandwidthDescription::Unbounded)
        .latency_topology(&[LatencyDescription::WithinPool(
            "ExamplePool",
            Distributions::Uniform(Jiffies(0), Jiffies(10)),
        )])
        .time_budget(Jiffies(100_000_000))
        .seed(5)
        .build();

    anykv::set::<usize>("pings", 0);
    anykv::set::<usize>("pongs", 0);

    let start = Instant::now();
    sim.run();
    let elapsed = start.elapsed();

    println!(
        "Done, elapsed: {:?}. Pings sent: {}, Pongs sent: {}",
        elapsed,
        anykv::get::<usize>("pings"),
        anykv::get::<usize>("pongs"),
    );
}
