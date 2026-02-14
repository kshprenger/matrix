use dscale::{global::anykv, *};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct BroadcastMessage {
    pub data: u64,
}

impl Message for BroadcastMessage {}

#[derive(Default)]
pub struct BroadcastProcess {}

impl ProcessHandle for BroadcastProcess {
    fn start(&mut self) {
        // Process with rank 1 starts the broadcast
        if rank() == 1 {
            schedule_timer_after(Jiffies(100));
        }
    }

    fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
        let msg = message.as_type::<BroadcastMessage>();
        debug_process!("Received broadcast from {}: data={}", from, msg.data);

        assert_eq!(msg.data, 42);

        anykv::modify::<usize>("broadcast_received", |x| *x += 1);
    }

    fn on_timer(&mut self, _id: TimerId) {
        debug_process!("Broadcasting value 42");
        broadcast(BroadcastMessage { data: 42 });
        schedule_timer_after(Jiffies(100));
    }
}
