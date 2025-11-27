use std::time::Instant;

use simulator::{Jiffies, Message, ProcessHandle, ProcessId, SimulationBuilder};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
struct ExampleMessage {}

impl Message for ExampleMessage {
    fn virtual_size(&self) -> usize {
        69
    }
}

struct ExampleProcess {}

impl ExampleProcess {
    fn new() -> Self {
        Self {}
    }
}

impl ProcessHandle<ExampleMessage> for ExampleProcess {
    fn bootstrap(
        &mut self,
        assigned_id: ProcessId,
        outgoing: &mut simulator::OutgoingMessages<ExampleMessage>,
    ) {
        outgoing.send_self(ExampleMessage {});
    }

    fn on_message(
        &mut self,
        from: ProcessId,
        message: ExampleMessage,
        outgoing: &mut simulator::OutgoingMessages<ExampleMessage>,
    ) {
        outgoing.send_self(ExampleMessage {});
    }
}

fn main() {
    let start = Instant::now();

    let m = SimulationBuilder::new_with_process_factory(|| ExampleProcess::new())
        .with_network_bandwidth(simulator::BandwidthType::Unbounded)
        .with_max_network_latency(Jiffies(2))
        .with_max_steps(Jiffies(100_000))
        .with_process_count(200)
        .with_seed(5)
        .build()
        .run();

    println!(
        "Done, events: {}, elapsed: {:?}",
        m.events_total,
        start.elapsed()
    )
}
