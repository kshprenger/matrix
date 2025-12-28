#![allow(non_snake_case)]

use std::time::Instant;

use simulator::{
    time::{Jiffies, timer::TimerId},
    *,
};

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
    timer_id: TimerId,
}

impl ExampleProcess {
    fn New() -> Self {
        Self {
            self_id: 0,
            timer_id: 0,
        }
    }
}

impl ProcessHandle for ExampleProcess {
    fn Bootstrap(&mut self, configuration: Configuration) {
        self.self_id = configuration.assigned_id;
        if configuration.assigned_id == 1 {
            self.timer_id = ScheduleTimerAfter(Jiffies(100));
        }
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        assert!(message.Is::<ExampleMessage>());
        let m = message.As::<ExampleMessage>();

        if from == 1 && self.self_id == 2 {
            assert!(*m == ExampleMessage::Ping);
            Debug!("Sending Pong");
            SendTo(1, ExampleMessage::Pong);
            return;
        }

        if from == 2 && self.self_id == 1 {
            assert!(*m == ExampleMessage::Pong);
            Debug!("Sending Ping");
            SendTo(2, ExampleMessage::Ping);
            return;
        }
    }

    fn OnTimer(&mut self, id: TimerId) {
        assert!(id == self.timer_id);
        SendTo(2, ExampleMessage::Ping);
    }
}

fn main() {
    let start = Instant::now();

    SimulationBuilder::NewFromFactory(|| Box::new(ExampleProcess::New()))
        .NICBandwidth(simulator::BandwidthType::Unbounded)
        .MaxLatency(Jiffies(10))
        .MaxTime(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!("Done, events elapsed: {:?}", start.elapsed());

    let start = Instant::now();

    SimulationBuilder::NewFromFactory(|| Box::new(ExampleProcess::New()))
        .NICBandwidth(simulator::BandwidthType::Bounded(5))
        .MaxLatency(Jiffies(10))
        .MaxTime(Jiffies(100_000_000))
        .ProcessInstances(2)
        .Seed(5)
        .Build()
        .Run();

    println!("Done, events: elapsed: {:?}", start.elapsed());
}
