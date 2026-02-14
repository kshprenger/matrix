use dscale::{global::anykv, *};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum PingPongMessage {
    Ping,
    Pong,
}

impl Message for PingPongMessage {}

#[derive(Default)]
pub struct PingPongProcess {}

impl ProcessHandle for PingPongProcess {
    fn start(&mut self) {
        if rank() == 1 {
            send_to(2, PingPongMessage::Ping);
        }
    }

    fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
        let m = message.as_type::<PingPongMessage>();

        if from == 1 && rank() == 2 {
            assert!(*m == PingPongMessage::Ping);
            debug_process!("Sending Pong");
            anykv::modify::<usize>("pongs", |p| *p += 1);
            send_to(1, PingPongMessage::Pong);
            return;
        }

        if from == 2 && rank() == 1 {
            assert!(*m == PingPongMessage::Pong);
            debug_process!("Sending Ping");
            anykv::modify::<usize>("pings", |p| *p += 1);
            send_to(2, PingPongMessage::Ping);
            return;
        }
    }

    fn on_timer(&mut self, _id: TimerId) {}
}
