use std::time::Instant;

use dscale::{global::anykv, *};
use examples::bandwidth::{Receiver, Sender};

fn main() {
    println!("=== Bandwidth Example ===\n");

    let unbounded_count = run_unbounded();
    println!("Unbounded: messages received = {}\n", unbounded_count);

    let bounded_count = run_bounded();
    println!("Bounded: messages received = {}\n", bounded_count);

    // Assert that bounded receives fewer messages due to bandwidth constraints
    assert!(
        bounded_count < unbounded_count,
        "Bounded ({}) should receive fewer messages than unbounded ({})",
        bounded_count,
        unbounded_count
    );
}

fn run_unbounded() -> usize {
    anykv::set::<usize>("messages_sent", 0);
    anykv::set::<usize>("messages_received", 0);

    let mut sim = SimulationBuilder::default()
        .add_pool::<Sender>("Senders", 1)
        .add_pool::<Receiver>("Receivers", 1)
        .nic_bandwidth(BandwidthDescription::Unbounded)
        .latency_topology(&[LatencyDescription::BetweenPools(
            "Senders",
            "Receivers",
            Distributions::Uniform(Jiffies(10), Jiffies(10)),
        )])
        .time_budget(Jiffies(10_000))
        .seed(42)
        .build();

    let start = Instant::now();
    sim.run();
    let elapsed = start.elapsed();

    let sent = anykv::get::<usize>("messages_sent");
    let received = anykv::get::<usize>("messages_received");
    println!(
        "  Elapsed: {:?}, sent: {}, received: {}",
        elapsed, sent, received
    );

    received
}

fn run_bounded() -> usize {
    anykv::set::<usize>("messages_sent", 0);
    anykv::set::<usize>("messages_received", 0);

    let mut sim = SimulationBuilder::default()
        .add_pool::<Sender>("Senders", 1)
        .add_pool::<Receiver>("Receivers", 1)
        // Very low bandwidth: 1 byte per jiffy (messages will queue up)
        .nic_bandwidth(BandwidthDescription::Bounded(1))
        .latency_topology(&[LatencyDescription::BetweenPools(
            "Senders",
            "Receivers",
            Distributions::Uniform(Jiffies(10), Jiffies(10)),
        )])
        .time_budget(Jiffies(10_000))
        .seed(42)
        .build();

    let start = Instant::now();
    sim.run();
    let elapsed = start.elapsed();

    let sent = anykv::get::<usize>("messages_sent");
    let received = anykv::get::<usize>("messages_received");
    println!(
        "  Elapsed: {:?}, sent: {}, received: {}",
        elapsed, sent, received
    );

    received
}
