#![allow(non_snake_case)]

use matrix::{global::anykv, *};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct BroadcastMessage {
    pub data: u64,
}

impl Message for BroadcastMessage {
    fn VirtualSize(&self) -> usize {
        0
    }
}

#[derive(Default)]
pub struct BroadcastProcess {}

impl ProcessHandle for BroadcastProcess {
    fn Start(&mut self) {
        // Process with Rank 1 starts the broadcast
        if Rank() == 1 {
            ScheduleTimerAfter(Jiffies(100));
        }
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        let msg = message.As::<BroadcastMessage>();
        Debug!("Received broadcast from {}: data={}", from, msg.data);

        // Verify content
        assert_eq!(msg.data, 42);

        // Increment a counter for verification in the main test
        anykv::Modify::<usize>("broadcast_received", |x| *x += 1);
    }

    fn OnTimer(&mut self, _id: TimerId) {
        Debug!("Broadcasting value 42");
        Broadcast(BroadcastMessage { data: 42 });
        ScheduleTimerAfter(Jiffies(100));
    }
}
