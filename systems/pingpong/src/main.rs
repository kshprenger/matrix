#![allow(non_snake_case)]

use std::time::Instant;

use matrix::*;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum PingPongMessage {
    Ping,
    Pong,
}

impl Message for PingPongMessage {
    fn VirtualSize(&self) -> usize {
        match self {
            PingPongMessage::Ping => 50,
            PingPongMessage::Pong => 100,
        }
    }
}

struct ExampleProcess {
    timer_id: TimerId,
}

impl ExampleProcess {
    fn New() -> Self {
        Self { timer_id: 0 }
    }
}

impl ProcessHandle for ExampleProcess {
    fn Bootstrap(&mut self, _configuration: Configuration) {
        if CurrentId() == 1 {
            assert!(ListPool("ExamplePool").len() == 2);
            assert!(ListPool("ExamplePool")[0] == 1);
            assert!(ListPool("ExamplePool")[1] == 2);
            self.timer_id = ScheduleTimerAfter(Jiffies(100));
        }
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        assert!(message.Is::<PingPongMessage>());
        let m = message.As::<PingPongMessage>();

        if from == 1 && CurrentId() == 2 {
            assert!(*m == PingPongMessage::Ping);
            Debug!("Sending Pong");
            anykv::Modify::<usize>("pongs", |p| *p += 1);
            SendTo(1, PingPongMessage::Pong);
            return;
        }

        if from == 2 && CurrentId() == 1 {
            assert!(*m == PingPongMessage::Pong);
            Debug!("Sending Ping");
            anykv::Modify::<usize>("pings", |p| *p += 1);
            SendTo(2, PingPongMessage::Ping);
            return;
        }
    }

    fn OnTimer(&mut self, id: TimerId) {
        assert!(id == self.timer_id);
        anykv::Modify::<usize>("pings", |p| *p += 1);
        SendTo(2, PingPongMessage::Ping);
    }
}

fn main() {
    let start = Instant::now();

    anykv::Set::<usize>("pings", 0);
    anykv::Set::<usize>("pongs", 0);

    SimulationBuilder::NewDefault()
        .AddPool("ExamplePool", 2, || ExampleProcess::New())
        .NICBandwidth(matrix::BandwidthType::Unbounded)
        .MaxLatency(Jiffies(10))
        .TimeBudget(Jiffies(100_000_000))
        .Seed(5)
        .Build()
        .Run();

    println!(
        "Done, elapsed: {:?}. Pings sent: {}, Pongs sent: {}",
        start.elapsed(),
        anykv::Get::<usize>("pings"),
        anykv::Get::<usize>("pongs"),
    );
}
