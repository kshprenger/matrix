#![allow(non_snake_case)]

use matrix::{global::anykv, *};

// This demo shows 2 data centers: in first one there s pingers processes,
// in the second one - pongers processes. Pingers send ping to a single random pong process and vice versa.

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum PingPongMessage {
    Ping,
    Pong,
}

impl Message for PingPongMessage {
    fn VirtualSize(&self) -> usize {
        0
    }
}

#[derive(Default)]
pub struct PingProcess {}

impl ProcessHandle for PingProcess {
    fn Start(&mut self) {
        SendRandomFromPool("Pongers", PingPongMessage::Ping);
        anykv::Modify::<usize>("pings", |p| *p += 1);
    }

    fn OnMessage(&mut self, _from: ProcessId, message: MessagePtr) {
        let m = message.As::<PingPongMessage>();
        match *m {
            PingPongMessage::Pong => {
                SendRandomFromPool("Pongers", PingPongMessage::Ping);
                anykv::Modify::<usize>("pings", |p| *p += 1);
            }
            _ => {}
        }
    }

    fn OnTimer(&mut self, _id: TimerId) {}
}

#[derive(Default)]
pub struct PongProcess {}

impl ProcessHandle for PongProcess {
    fn Start(&mut self) {}

    fn OnMessage(&mut self, _from: ProcessId, message: MessagePtr) {
        let m = message.As::<PingPongMessage>();
        match *m {
            PingPongMessage::Ping => {
                SendRandomFromPool("Pingers", PingPongMessage::Pong);
                anykv::Modify::<usize>("pongs", |p| *p += 1);
            }
            _ => {}
        }
    }

    fn OnTimer(&mut self, _id: TimerId) {}
}
