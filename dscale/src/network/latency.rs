use std::collections::BinaryHeap;
use std::rc::Rc;

use log::debug;

use crate::communication::{RoutedMessage, TimePriorityMessageQueue};
use crate::random::Randomizer;
use crate::topology::Topology;

pub(crate) struct LatencyQueue {
    topology: Rc<Topology>,
    randomizer: Randomizer,
    queue: TimePriorityMessageQueue,
}
impl LatencyQueue {
    pub(crate) fn New(randomizer: Randomizer, topology: Rc<Topology>) -> Self {
        Self {
            randomizer,
            topology,
            queue: BinaryHeap::new(),
        }
    }

    pub(crate) fn Push(&mut self, mut message: RoutedMessage) {
        debug!(
            "Arrival time before adding latency: {}",
            message.arrival_time
        );
        message.arrival_time += self.randomizer.RandomUsize(
            self.topology
                .GetDistribution(message.step.source, message.step.dest),
        );
        debug!(
            "Arrival time after adding random latency: {}",
            message.arrival_time
        );
        self.queue.push(std::cmp::Reverse(message));
    }

    pub(crate) fn Pop(&mut self) -> Option<RoutedMessage> {
        Some(self.queue.pop()?.0)
    }

    pub(crate) fn Peek(&self) -> Option<&RoutedMessage> {
        Some(&self.queue.peek()?.0)
    }
}
