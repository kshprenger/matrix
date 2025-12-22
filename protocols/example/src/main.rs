#![allow(non_snake_case)]

use std::time::Instant;

use simulator::*;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum ExampleMessage {
    Ping,
    Pong,
}

impl Message for ExampleMessage {
    fn VirtualSize(&self) -> usize {
        100
    }
}

struct ExampleProcess {
    self_id: ProcessId,
}

impl ExampleProcess {
    fn New() -> Self {
        Self { self_id: 0 }
    }
}

impl ProcessHandle for ExampleProcess {
    fn Bootstrap(&mut self, configuration: Configuration) {
        self.self_id = configuration.assigned_id;
        if configuration.assigned_id == 1 {
            SendTo(2, ExampleMessage::Ping);
        }
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        assert!(message.Is::<ExampleMessage>());
        let m = message.As::<ExampleMessage>();

        if from == 1 && self.self_id == 2 {
            assert!(*m == ExampleMessage::Ping);
            SendTo(1, ExampleMessage::Pong);
            return;
        }

        if from == 2 && self.self_id == 1 {
            assert!(*m == ExampleMessage::Pong);
            SendTo(2, ExampleMessage::Ping);
            return;
        }
    }
}

fn main() {
    let start = Instant::now();

    let m = SimulationBuilder::NewFromFactory(|| ExampleProcess::New())
        .NetworkBandwidth(simulator::BandwidthType::Unbounded)
        .MaxLatency(Jiffies(10))
        .MaxTime(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!(
        "Done, events: {}, elapsed: {:?}",
        m.events_total,
        start.elapsed()
    );

    let start = Instant::now();

    let m = SimulationBuilder::NewFromFactory(|| ExampleProcess::New())
        .NetworkBandwidth(simulator::BandwidthType::Bounded(5))
        .MaxLatency(Jiffies(10))
        .MaxTime(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!(
        "Done, events: {}, elapsed: {:?}",
        m.events_total,
        start.elapsed()
    );
}
