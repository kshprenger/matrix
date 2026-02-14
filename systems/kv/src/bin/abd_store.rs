use dscale::{global::anykv, *};
use kv::abd_store::{
    Replica,
    client::{Client, ExecutionHistory},
    lin_checker::check_linearizable,
    types::{CLIENT_POOL_NAME, REPLICA_POOL_NAME},
};

fn main() {
    // 1 jiffy == 1ms
    let mut sim = SimulationBuilder::default()
        .add_pool::<Replica>(REPLICA_POOL_NAME, 10)
        .add_pool::<Client>(CLIENT_POOL_NAME, 4)
        .time_budget(Jiffies(5000))
        .latency_topology(&[
            LatencyDescription::WithinPool(
                REPLICA_POOL_NAME,
                Distributions::Uniform(Jiffies(0), Jiffies(10)),
            ),
            LatencyDescription::WithinPool(
                CLIENT_POOL_NAME,
                Distributions::Uniform(Jiffies(0), Jiffies(545)),
            ),
            LatencyDescription::BetweenPools(
                CLIENT_POOL_NAME,
                REPLICA_POOL_NAME,
                Distributions::Uniform(Jiffies(0), Jiffies(1212)),
            ),
        ])
        .seed(5444)
        .build();

    anykv::set::<ExecutionHistory>("linearizable_history", ExecutionHistory::new());

    sim.run();

    println!(
        "{:<8} | {:<12} | {:<8} | {:<12} | {:<12}",
        "CLIENT ID", "OPERATION", "RESULT", "START", "END"
    );
    println!("{}", "-".repeat(75));

    let history = anykv::get::<ExecutionHistory>("linearizable_history");

    for el in anykv::get::<ExecutionHistory>("linearizable_history") {
        let result = el
            .result
            .map(|v| v.to_string())
            .unwrap_or_else(|| "Ack".to_string());
        println!(
            "{:<8} | {:<12} | {:<8} | {:<12} | {:<12}",
            el.client, el.operation, result, el.start, el.end
        );
    }

    assert!(check_linearizable(&history));
}
