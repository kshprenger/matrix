#![allow(non_snake_case)]

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
    fn Start(&mut self) {
        if Rank() == 1 {
            SendTo(2, PingPongMessage::Ping);
        }
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        let m = message.As::<PingPongMessage>();

        if from == 1 && Rank() == 2 {
            assert!(*m == PingPongMessage::Ping);
            Debug!("Sending Pong");
            anykv::Modify::<usize>("pongs", |p| *p += 1);
            SendTo(1, PingPongMessage::Pong);
            return;
        }

        if from == 2 && Rank() == 1 {
            assert!(*m == PingPongMessage::Pong);
            Debug!("Sending Ping");
            anykv::Modify::<usize>("pings", |p| *p += 1);
            SendTo(2, PingPongMessage::Ping);
            return;
        }
    }

    fn OnTimer(&mut self, _id: TimerId) {}
}
