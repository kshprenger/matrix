use std::time::Instant;

use examples::timer_demo::LazyPingPong;
use matrix::{global::anykv, *};

fn main() {
    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<LazyPingPong>("TimerDemoPool", 2)
        .NICBandwidth(BandwidthDescription::Unbounded)
        .LatencyTopology(&[LatencyDescription::WithinPool(
            "TimerDemoPool",
            Distributions::Uniform(Jiffies(10), Jiffies(50)),
        )])
        .TimeBudget(Jiffies(10_000))
        .Seed(42)
        .Build();

    anykv::Set::<usize>("heartbeats", 0);
    anykv::Set::<usize>("pings_received", 0);
    anykv::Set::<usize>("pongs_received", 0);

    let start = Instant::now();
    sim.Run();
    let elapsed = start.elapsed();

    let heartbeats = anykv::Get::<usize>("heartbeats");
    let pings = anykv::Get::<usize>("pings_received");
    let pongs = anykv::Get::<usize>("pongs_received");

    println!();
    println!("Simulation completed in: {:?}", elapsed);
    println!("Heartbeats: {}", heartbeats);
    println!("Pings received: {}", pings);
    println!("Pongs received: {}", pongs);

    assert_eq!(pings, 5);
    assert_eq!(pongs, 5);
    assert_eq!(heartbeats, 19);
}
