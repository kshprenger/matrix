use kv::abd_store::{
    Replica,
    client::Client,
    types::{CLIENT_POOL_NAME, REPLICA_POOL_NAME},
};
use matrix::*;

fn main() {
    // 1 jiffy == 1ms
    let sim = SimulationBuilder::NewDefault()
        .AddPool(REPLICA_POOL_NAME, 3, Replica::New)
        .AddPool(CLIENT_POOL_NAME, 1, Client::New)
        .TimeBudget(Jiffies(500))
        .MaxLatency(Jiffies(0))
        .Seed(123)
        .Build();

    sim.Run();
}
