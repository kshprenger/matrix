use std::collections::BinaryHeap;

use log::debug;

use crate::{
    communication::{RoutedMessage, TimePriorityMessageQueue},
    network_condition::LatencyQueue,
    time::Jiffies,
};

#[derive(Clone, Copy)]
pub enum BandwidthType {
    Unbounded,
    Bounded(usize), // Bytes per Jiffy
}

#[derive(Clone)]
pub(crate) enum BandwidthQueueOptions {
    MessageArrivedByLatency,
    None,
    Some(RoutedMessage),
}

pub(crate) struct BandwidthQueue {
    bandwidth: usize,
    global_queue: LatencyQueue,
    current_buffers_sizes: Vec<usize>,
    merged_fifo_buffers: TimePriorityMessageQueue,
}

impl BandwidthQueue {
    pub(crate) fn New(
        bandwidth_type: BandwidthType,
        proc_num: usize,
        global_queue: LatencyQueue,
    ) -> Self {
        let bandwidth = match bandwidth_type {
            BandwidthType::Unbounded => usize::MAX,
            BandwidthType::Bounded(bound) => bound,
        };

        Self {
            bandwidth,
            global_queue,
            current_buffers_sizes: vec![0; proc_num + 1],
            merged_fifo_buffers: BinaryHeap::new(),
        }
    }

    pub(crate) fn Push(&mut self, message: RoutedMessage) {
        debug!("Submitted message with base time: {}", message.arrival_time);
        self.global_queue.Push(message);
    }

    pub(crate) fn Pop(&mut self) -> BandwidthQueueOptions {
        let closest_arriving_message = self.global_queue.Peek();
        let closest_squeezing_message = self.merged_fifo_buffers.peek();

        match (closest_arriving_message, closest_squeezing_message) {
            (None, None) => BandwidthQueueOptions::None,
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
}

impl BandwidthQueue {
    fn MoveMessageFromLatencyQueueToBuffers(&mut self) {
        debug!("Moving message from latency queue to buffers");
        let mut message = self
            .global_queue
            .Pop()
            .expect("Global queue should not be empty");
        self.current_buffers_sizes[message.step.dest] += message.step.message.VirtualSize();
        debug!(
            "New process {} buffer's size: {}",
            message.step.dest, self.current_buffers_sizes[message.step.dest]
        );
        debug!(
            "Message arrival time before bandwidth adjustment: {}",
            message.arrival_time
        );
        message.arrival_time +=
            Jiffies(self.current_buffers_sizes[message.step.dest] / self.bandwidth);
        debug!(
            "Message arrival time after bandwidth adjustment: {}",
            message.arrival_time
        );
        self.merged_fifo_buffers.push(std::cmp::Reverse(message));
    }

    fn DeliverFromBuffer(&mut self) -> BandwidthQueueOptions {
        let message = self
            .merged_fifo_buffers
            .pop()
            .expect("All buffers should not be empty")
            .0;
        self.current_buffers_sizes[message.step.dest] -= message.step.message.VirtualSize();
        debug!(
            "New process {} buffer's size: {}",
            message.step.dest, self.current_buffers_sizes[message.step.dest]
        );
        BandwidthQueueOptions::Some(message)
    }

    fn DeliverFromLatencyQueue(&mut self) -> BandwidthQueueOptions {
        self.MoveMessageFromLatencyQueueToBuffers();
        BandwidthQueueOptions::MessageArrivedByLatency
    }
}
