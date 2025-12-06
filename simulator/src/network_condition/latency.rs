use std::collections::BinaryHeap;

use crate::communication::{RoutedMessage, TimePriorityMessageQueue};
use crate::{Message, ProcessId};
use crate::{random::Randomizer, time::Jiffies};

pub(crate) struct LatencyQueue<M: Message> {
    randomizer: Randomizer,
    max_latency: Jiffies,
    queue: TimePriorityMessageQueue<M>,
}

impl<M: Message> LatencyQueue<M> {
    pub(crate) fn new(randomizer: Randomizer, max_latency: Jiffies) -> Self {
        Self {
            randomizer,
            max_latency,
            queue: BinaryHeap::new(),
        }
    }

    pub(crate) fn push(&mut self, mut message: RoutedMessage<M>) {
        message.0 += self
            .randomizer
            .random_from_range_uniform(0, self.max_latency.0);
        self.queue.push(std::cmp::Reverse(message));
    }

    pub(crate) fn pop(&mut self) -> Option<RoutedMessage<M>> {
        Some(self.queue.pop()?.0)
    }

    pub(crate) fn peek(&mut self) -> Option<&RoutedMessage<M>> {
        Some(&self.queue.peek()?.0)
    }
}
