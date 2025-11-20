use priority_queue::PriorityQueue;

use crate::{process::ProcessId, time::Jiffies};

pub type EventId = usize;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Event {
    pub id: EventId,
    pub event_type: EventType,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EventType {
    Timeout,
    Message(Message),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Message {
    source: ProcessId,
    payload: bytes::Bytes,
}

/// (Jiffies, Event) <=> At speciffied timestamp event will be delivered
pub type EventDeliveryQueue = PriorityQueue<Event, Jiffies>;
