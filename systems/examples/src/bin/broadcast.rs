use std::time::Instant;

use dscale::{global::anykv, *};
use examples::broadcast::BroadcastProcess;

fn main() {
    anykv::Set::<usize>("broadcast_received", 0);

    let mut sim = SimulationBuilder::NewDefault()
        .AddPool::<BroadcastProcess>("BroadcastPool", 5)
        .NICBandwidth(BandwidthDescription::Unbounded)
        .LatencyTopology(&[LatencyDescription::WithinPool(
            "BroadcastPool",
            Distributions::Uniform(Jiffies(0), Jiffies(10)),
        )])
        .TimeBudget(Jiffies(100_0000))
        .Seed(123)
        .Build();

    let start = Instant::now();
    sim.Run();
    let elapsed = start.elapsed();

    let received_count = anykv::Get::<usize>("broadcast_received");
    println!(
        "Done, elapsed: {:?}. Broadcast messages received: {}",
        elapsed, received_count
    );
    assert_eq!(received_count, 49995);
}
