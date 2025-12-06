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

// (Arrival time, source, dest, message)
pub type RoutedMessage<M> = (Jiffies, (ProcessId, ProcessId, M));

pub type TimePriorityMessageQueue<M> = std::collections::BinaryHeap<Reverse<RoutedMessage<M>>>;
