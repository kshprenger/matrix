use std::time::Instant;

use dscale::{global::anykv, *};
use examples::broadcast::BroadcastProcess;

fn main() {
    anykv::set::<usize>("broadcast_received", 0);

    let mut sim = SimulationBuilder::default()
        .add_pool::<BroadcastProcess>("BroadcastPool", 5)
        .nic_bandwidth(BandwidthDescription::Unbounded)
        .latency_topology(&[LatencyDescription::WithinPool(
            "BroadcastPool",
            Distributions::Uniform(Jiffies(0), Jiffies(10)),
        )])
        .time_budget(Jiffies(100_0000))
        .seed(123)
        .build();

    let start = Instant::now();
    sim.run();
    let elapsed = start.elapsed();

    let received_count = anykv::get::<usize>("broadcast_received");
    println!(
        "Done, elapsed: {:?}. Broadcast messages received: {}",
        elapsed, received_count
    );
    assert_eq!(received_count, 49995);
}
