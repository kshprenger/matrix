use std::collections::BinaryHeap;

use log::debug;

use crate::{
    communication::{RoutedMessage, TimePriorityMessageQueue},
    network::LatencyQueue,
    now,
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
    pub(crate) fn new(
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

    pub(crate) fn push(&mut self, message: RoutedMessage) {
        debug!("Submitted message with base time: {}", message.arrival_time);
        self.global_queue.push(message);
    }

    pub(crate) fn pop(&mut self) -> Option<RoutedMessage> {
        let closest_arriving_message = self.global_queue.peek();
        let closest_squeezing_message = self.merged_fifo_buffers.peek();

        match (closest_arriving_message, closest_squeezing_message) {
            (None, None) => None,
            (Some(_), None) => self.deliver_from_latency_queue(),
            (None, Some(_)) => self.deliver_from_buffer(),
            (Some(l_message), Some(b_message)) => {
                if l_message.arrival_time <= b_message.0.arrival_time {
                    self.deliver_from_latency_queue()
                } else {
                    self.deliver_from_buffer()
                }
            }
        }
    }

    pub(crate) fn peek_closest(&self) -> Option<Jiffies> {
        let closest_arriving_message = self.global_queue.peek();
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
    fn move_message_from_latency_queue_to_buffers(&mut self) {
        debug!("Moving message from latency queue to buffers");
        let mut message = self
            .global_queue
            .pop()
            .expect("Global queue should not be empty");

        if self.bandwidth == usize::MAX {
            self.merged_fifo_buffers.push(std::cmp::Reverse(message));
        } else {
            let new_total =
                self.total_pased[message.step.dest] + message.step.message.virtual_size();

            if new_total > now().0 * self.bandwidth {
                message.arrival_time = Jiffies(new_total / self.bandwidth); // > now()
            }

            self.merged_fifo_buffers.push(std::cmp::Reverse(message));
        }
    }

    fn deliver_from_buffer(&mut self) -> Option<RoutedMessage> {
        let message = self
            .merged_fifo_buffers
            .pop()
            .expect("All buffers should not be empty")
            .0;
        self.total_pased[message.step.dest] += message.step.message.virtual_size();
        Some(message)
    }

    fn deliver_from_latency_queue(&mut self) -> Option<RoutedMessage> {
        self.move_message_from_latency_queue_to_buffers();
        None
    }
}
