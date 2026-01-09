use kv::abd_store::{
    Replica,
    client::{Client, ExecutionHistory},
    lin_checker::CheckLinearizable,
    types::{CLIENT_POOL_NAME, REPLICA_POOL_NAME},
};
use matrix::{global::anykv, *};

fn main() {
    // 1 jiffy == 1ms
    let sim = SimulationBuilder::NewDefault()
        .AddPool::<Replica>(REPLICA_POOL_NAME, 10)
        .AddPool::<Client>(CLIENT_POOL_NAME, 4)
        .TimeBudget(Jiffies(5000))
        .MaxLatency(Jiffies(0))
        .Seed(13123123)
        .Build();

    anykv::Set::<ExecutionHistory>("linearizable_history", ExecutionHistory::new());

    sim.Run();

    println!(
        "{:<8} | {:<12} | {:<8} | {:<12} | {:<12}",
        "CLIENT ID", "OPERATION", "RESULT", "START", "END"
    );
    println!("{}", "-".repeat(75));

    let history = anykv::Get::<ExecutionHistory>("linearizable_history");

    for el in anykv::Get::<ExecutionHistory>("linearizable_history") {
        let result = el
            .result
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<8} | {:<12} | {:<8} | {:<12} | {:<12}",
            el.client, el.operation, result, el.start, el.end
        );
    }

    assert!(CheckLinearizable(&history));
}
