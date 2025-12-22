use std::collections::BinaryHeap;

use log::debug;

use crate::communication::{RoutedMessage, TimePriorityMessageQueue};
use crate::{random::Randomizer, time::Jiffies};

pub(crate) struct LatencyQueue {
    randomizer: Randomizer,
    max_latency: Jiffies,
    queue: TimePriorityMessageQueue,
}
impl LatencyQueue {
    pub(crate) fn New(randomizer: Randomizer, max_latency: Jiffies) -> Self {
        Self {
            randomizer,
            max_latency,
            queue: BinaryHeap::new(),
        }
    }

    pub(crate) fn Push(&mut self, mut message: RoutedMessage) {
        debug!(
            "Arrival time before adding latency: {}",
            message.arrival_time
        );
        message.arrival_time += self.randomizer.RandomFromRange(0, self.max_latency.0);
        debug!(
            "Arrival time after adding random latency: {}",
            message.arrival_time
        );
        self.queue.push(std::cmp::Reverse(message));
    }

    pub(crate) fn Pop(&mut self) -> Option<RoutedMessage> {
        Some(self.queue.pop()?.0)
    }

    pub(crate) fn Peek(&mut self) -> Option<&RoutedMessage> {
        Some(&self.queue.peek()?.0)
    }
}
