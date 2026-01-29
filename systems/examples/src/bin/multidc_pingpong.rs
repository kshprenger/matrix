use std::time::Instant;

use examples::multidc_pingpong::{PingProcess, PongProcess};
use matrix::{global::anykv, *};

fn main() {
    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<PingProcess>("Pingers", 3)
        .AddPool::<PongProcess>("Pongers", 2)
        .NICBandwidth(BandwidthDescription::Unbounded)
        .LatencyTopology(&[
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
        .TimeBudget(Jiffies(100_000))
        .Seed(5)
        .Build();

    anykv::Set::<usize>("pings", 0);
    anykv::Set::<usize>("pongs", 0);

    let start = Instant::now();
    sim.Run();
    let elapsed = start.elapsed();

    let pings = anykv::Get::<usize>("pings");
    let pongs = anykv::Get::<usize>("pongs");

    println!(
        "Done, elapsed: {:?}. Pings sent: {}, Pongs sent: {}",
        elapsed, pings, pongs,
    );

    assert_eq!(pings, 9380);
    assert_eq!(pings, 9380);
}
