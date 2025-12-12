#![allow(non_snake_case)]

mod dag;

use std::rc::Rc;

use simulator::*;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
enum BullsharkMessage {}

impl Message for BullsharkMessage {
    fn VirtualSize(&self) -> usize {
        todo!()
    }
}

struct Vertex {}

struct Bullshark {
    dag: Vec<Vec<Rc<Vertex>>>,
}

impl Bullshark {
    fn New() -> Self {
        Self {}
    }
}

impl ProcessHandle<BullsharkMessage> for Bullshark {
    fn Bootstrap(
        &mut self,
        assigned_id: ProcessId,
        outgoing: &mut simulator::OutgoingMessages<BullsharkMessage>,
    ) {
        todo!()
    }

    fn OnMessage(
        &mut self,
        from: ProcessId,
        message: BullsharkMessage,
        outgoing: &mut simulator::OutgoingMessages<BullsharkMessage>,
    ) {
        todo!();
    }
}

impl Bullshark {
    fn TryAdvanceRound(&mut self) {
        todo!()
    }
}

fn main() {
    todo!()
}
