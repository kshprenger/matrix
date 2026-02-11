use std::collections::BinaryHeap;

use log::debug;

use crate::{
    Now,
    communication::{RoutedMessage, TimePriorityMessageQueue},
    network::LatencyQueue,
    time::Jiffies,
};

#[derive(Clone, Copy)]
pub enum BandwidthDescription {
    Unbounded,
    Bounded(usize), // Bytes per Jiffy
}

pub(crate) struct BandwidthQueue {
    bandwidth: usize,
    global_queue: LatencyQueue,
    total_pased: Vec<usize>,
    merged_fifo_buffers: TimePriorityMessageQueue,
}

impl BandwidthQueue {
    pub(crate) fn New(
        bandwidth_type: BandwidthDescription,
        proc_num: usize,
        global_queue: LatencyQueue,
    ) -> Self {
        let bandwidth = match bandwidth_type {
            BandwidthDescription::Unbounded => usize::MAX,
            BandwidthDescription::Bounded(bound) => bound,
        };

        Self {
            bandwidth,
            global_queue,
            total_pased: vec![0; proc_num + 1],
            merged_fifo_buffers: BinaryHeap::new(),
        }
    }

    pub(crate) fn Push(&mut self, message: RoutedMessage) {
        debug!("Submitted message with base time: {}", message.arrival_time);
        self.global_queue.Push(message);
    }

    pub(crate) fn Pop(&mut self) -> Option<RoutedMessage> {
        let closest_arriving_message = self.global_queue.Peek();
        let closest_squeezing_message = self.merged_fifo_buffers.peek();

        match (closest_arriving_message, closest_squeezing_message) {
            (None, None) => None,
            (Some(_), None) => self.DeliverFromLatencyQueue(),
            (None, Some(_)) => self.DeliverFromBuffer(),
            (Some(l_message), Some(b_message)) => {
                if l_message.arrival_time <= b_message.0.arrival_time {
                    self.DeliverFromLatencyQueue()
                } else {
                    self.DeliverFromBuffer()
                }
            }
        }
    }

    pub(crate) fn GetAvgTotalPasedBytes(&self) -> usize {
        self.total_pased.iter().sum::<usize>() / self.total_pased.len()
    }

    pub(crate) fn PeekClosest(&self) -> Option<Jiffies> {
        let closest_arriving_message = self.global_queue.Peek();
        let closest_squeezing_message = self.merged_fifo_buffers.peek();

        match (closest_arriving_message, closest_squeezing_message) {
            (None, None) => None,
            (Some(m), None) => Some(m.arrival_time),
            (None, Some(m)) => Some(m.0.arrival_time),
            (Some(l_message), Some(b_message)) => {
                if l_message.arrival_time <= b_message.0.arrival_time {
                    Some(l_message.arrival_time)
                } else {
                    Some(b_message.0.arrival_time)
                }
            }
        }
    }
}

impl BandwidthQueue {
    fn MoveMessageFromLatencyQueueToBuffers(&mut self) {
        debug!("Moving message from latency queue to buffers");
        let mut message = self
            .global_queue
            .Pop()
            .expect("Global queue should not be empty");

        if self.bandwidth == usize::MAX {
            self.merged_fifo_buffers.push(std::cmp::Reverse(message));
        } else {
            let new_total =
                self.total_pased[message.step.dest] + message.step.message.VirtualSize();

            if new_total > Now().0 * self.bandwidth {
                message.arrival_time = Jiffies(new_total / self.bandwidth); // > Now()
            }

            self.merged_fifo_buffers.push(std::cmp::Reverse(message));
        }
    }

    fn DeliverFromBuffer(&mut self) -> Option<RoutedMessage> {
        let message = self
            .merged_fifo_buffers
            .pop()
            .expect("All buffers should not be empty")
            .0;
        self.total_pased[message.step.dest] += message.step.message.VirtualSize();
        Some(message)
    }

    fn DeliverFromLatencyQueue(&mut self) -> Option<RoutedMessage> {
        self.MoveMessageFromLatencyQueueToBuffers();
        None
    }
}
