use std::time::Instant;

use dscale::{global::anykv, *};
use examples::multidc_pingpong::{PingProcess, PongProcess};

fn main() {
    let mut sim = SimulationBuilder::default()
        .add_pool::<PingProcess>("Pingers", 3)
        .add_pool::<PongProcess>("Pongers", 2)
        .nic_bandwidth(BandwidthDescription::Unbounded)
        .latency_topology(&[
            LatencyDescription::WithinPool(
                "Pingers",
                Distributions::Uniform(Jiffies(0), Jiffies(10)),
            ),
            LatencyDescription::WithinPool(
                "Pongers",
                Distributions::Uniform(Jiffies(0), Jiffies(10)),
            ),
            LatencyDescription::BetweenPools(
                "Pingers",
                "Pongers",
                Distributions::Uniform(Jiffies(10), Jiffies(20)),
            ),
        ])
        .time_budget(Jiffies(100_000))
        .seed(5)
        .build();

    anykv::set::<usize>("pings", 0);
    anykv::set::<usize>("pongs", 0);

    let start = Instant::now();
    sim.run();
    let elapsed = start.elapsed();

    let pings = anykv::get::<usize>("pings");
    let pongs = anykv::get::<usize>("pongs");

    println!(
        "Done, elapsed: {:?}. Pings sent: {}, Pongs sent: {}",
        elapsed, pings, pongs,
    );

    assert_eq!(pings, 9380);
    assert_eq!(pings, 9380);
}
