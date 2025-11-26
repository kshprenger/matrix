use std::cmp::Reverse;

use crate::{process::ProcessId, time::Jiffies};

pub trait Message: Eq + PartialEq + Ord + PartialOrd + Clone {
    fn virtual_size(&self) -> usize;
}

pub enum Destination {
    Broadcast,
    To(ProcessId),
    SendSelf,
}

/// (Jiffies, ProcessId, Event) <=> At specified timestamp event will be delivered with source of ProcessId
pub type TimePriorityMessageQueue<M> =
    std::collections::BinaryHeap<Reverse<(Jiffies, (ProcessId, M))>>;
