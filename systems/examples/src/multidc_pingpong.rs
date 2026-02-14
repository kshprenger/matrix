use dscale::{global::anykv, *};

// This demo shows 2 data centers: in first one there are pingers processes,
// in the second one - pongers processes. Pingers send ping to a single random pong process and vice versa.

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Ping;
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Pong;

impl Message for Ping {}
impl Message for Pong {}

#[derive(Default)]
pub struct PingProcess {}

impl ProcessHandle for PingProcess {
    fn start(&mut self) {
        send_random_from_pool("Pongers", Ping);
        anykv::modify::<usize>("pings", |p| *p += 1);
    }

    fn on_message(&mut self, _from: ProcessId, message: MessagePtr) {
        let _ = message.is::<Pong>();
        send_random_from_pool("Pongers", Ping);
        anykv::modify::<usize>("pings", |p| *p += 1);
    }

    fn on_timer(&mut self, _id: TimerId) {}
}

#[derive(Default)]
pub struct PongProcess {}

impl ProcessHandle for PongProcess {
    fn start(&mut self) {}

    fn on_message(&mut self, _from: ProcessId, message: MessagePtr) {
        let _ = message.is::<Ping>();
        send_random_from_pool("Pingers", Pong);
        anykv::modify::<usize>("pongs", |p| *p += 1);
    }

    fn on_timer(&mut self, _id: TimerId) {}
}
