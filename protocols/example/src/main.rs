use std::time::Instant;

use simulator::{Jiffies, Message, ProcessHandle, ProcessId, SimulationBuilder};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum ExampleMessage {
    Ping,
    Pong,
}

impl Message for ExampleMessage {
    fn virtual_size(&self) -> usize {
        100
    }
}

struct ExampleProcess {
    self_id: ProcessId,
}

impl ExampleProcess {
    fn new() -> Self {
        Self { self_id: 0 }
    }
}

impl ProcessHandle<ExampleMessage> for ExampleProcess {
    fn bootstrap(
        &mut self,
        assigned_id: ProcessId,
        outgoing: &mut simulator::OutgoingMessages<ExampleMessage>,
    ) {
        self.self_id = assigned_id;
        if assigned_id == 1 {
            outgoing.send_to(2, ExampleMessage::Ping);
        }
    }

    fn on_message(
        &mut self,
        from: ProcessId,
        message: ExampleMessage,
        outgoing: &mut simulator::OutgoingMessages<ExampleMessage>,
    ) {
        if from == 1 && self.self_id == 2 {
            assert!(message == ExampleMessage::Ping);
            outgoing.send_to(1, ExampleMessage::Pong);
            return;
        }

        if from == 2 && self.self_id == 1 {
            assert!(message == ExampleMessage::Pong);
            outgoing.send_to(2, ExampleMessage::Ping);
            return;
        }
    }
}

fn main() {
    let start = Instant::now();

    let m = SimulationBuilder::new_with_process_factory(|| ExampleProcess::new())
        .with_network_bandwidth(simulator::BandwidthType::Unbounded)
        .with_max_network_latency(Jiffies(10))
        .with_max_simulation_time(Jiffies(100_000_000))
        .with_process_count(2)
        .with_seed(5)
        .build()
        .run();

    println!(
        "Done, events: {}, elapsed: {:?}",
        m.events_total,
        start.elapsed()
    );

    let start = Instant::now();

    let m = SimulationBuilder::new_with_process_factory(|| ExampleProcess::new())
        .with_network_bandwidth(simulator::BandwidthType::Bounded(5))
        .with_max_network_latency(Jiffies(10))
        .with_max_simulation_time(Jiffies(100_000_000))
        .with_process_count(2)
        .with_seed(5)
        .build()
        .run();

    println!(
        "Done, events: {}, elapsed: {:?}",
        m.events_total,
        start.elapsed()
    );
}
