use dscale::{global::anykv, *};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct DataMessage {
    pub real_payload: u64,
}

impl Message for DataMessage {
    fn virtual_size(&self) -> usize {
        1000
    }
}

#[derive(Default)]
pub struct Sender {}

impl ProcessHandle for Sender {
    fn start(&mut self) {
        // Start sending immediately
        schedule_timer_after(Jiffies(1));
    }

    fn on_message(&mut self, _from: ProcessId, _message: MessagePtr) {
        // Sender doesn't receive messages
    }

    fn on_timer(&mut self, _id: TimerId) {
        send_to(2, DataMessage { real_payload: 42 });
        anykv::modify::<usize>("messages_sent", |x| *x += 1);
        schedule_timer_after(Jiffies(1));
    }
}

#[derive(Default)]
pub struct Receiver {}

impl ProcessHandle for Receiver {
    fn start(&mut self) {}

    fn on_message(&mut self, _from: ProcessId, message: MessagePtr) {
        let _ = message.as_type::<DataMessage>();
        anykv::modify::<usize>("messages_received", |x| *x += 1);
    }

    fn on_timer(&mut self, _id: TimerId) {}
}
