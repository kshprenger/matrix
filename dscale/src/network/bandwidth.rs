//! Network bandwidth configuration and simulation.
//!
//! This module provides bandwidth modeling for DScale simulations, allowing
//! realistic simulation of network bottlenecks and transmission delays.
//! Bandwidth constraints are applied per-process to model individual network
//! interface limitations.

use std::collections::BinaryHeap;

use log::debug;

use crate::{
    message::{RoutedMessage, TimePriorityMessageQueue},
    network::LatencyQueue,
    now,
    time::Jiffies,
};

/// Describes bandwidth constraints for network interfaces in the simulation.
///
/// `BandwidthDescription` defines how network bandwidth limitations are applied
/// to each process in the simulation. Bandwidth constraints affect message
/// transmission rates and can create realistic network bottlenecks that impact
/// the behavior of distributed systems.
///
/// # Bandwidth Modeling
///
/// The bandwidth simulation works by:
/// 1. Messages specify their size through [`Message::virtual_size`]
/// 2. The network calculates transmission time based on bandwidth limits
/// 3. Messages are delayed if they would exceed the available bandwidth
/// 4. Each process has its own bandwidth budget that replenishes over time
///
/// # Per-Process Application
///
/// Bandwidth limits are applied individually to each process, simulating
/// separate network interfaces. This means that:
/// - Each process can send up to the bandwidth limit per time unit
/// - Processes don't share bandwidth with each other
/// - Bandwidth exhaustion affects only the sending process
///
/// # Time Units
///
/// Bandwidth is measured in bytes per [`Jiffy`], where a Jiffy is the basic
/// unit of simulation time. The actual real-world time represented by a Jiffy
/// depends on your simulation's context and interpretation.
///
/// # Examples
///
/// ## Unlimited Bandwidth
///
/// ```rust
/// use dscale::{SimulationBuilder, BandwidthDescription};
///
/// let simulation = SimulationBuilder::default()
///     .add_pool::<MyProcess>("nodes", 3)
///     .nic_bandwidth(BandwidthDescription::Unbounded) // No bandwidth limits
///     .build();
/// # struct MyProcess;
/// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
/// # impl dscale::ProcessHandle for MyProcess {
/// #     fn start(&mut self) {}
/// #     fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {}
/// #     fn on_timer(&mut self, id: dscale::TimerId) {}
/// # }
/// ```
///
/// ## Limited Bandwidth Simulation
///
/// ```rust
/// use dscale::{SimulationBuilder, BandwidthDescription, Message};
///
/// struct LargeDataMessage {
///     data: Vec<u8>,
/// }
///
/// impl Message for LargeDataMessage {
///     fn virtual_size(&self) -> usize {
///         self.data.len() + 64 // Data + header overhead
///     }
/// }
///
/// let simulation = SimulationBuilder::default()
///     .add_pool::<MyProcess>("servers", 2)
///     .nic_bandwidth(BandwidthDescription::Bounded(1000)) // 1KB per jiffy
///     .build();
///
/// // With this configuration:
/// // - Messages up to 1000 bytes transmit in 1 jiffy
/// // - Larger messages take proportionally longer
/// // - Multiple messages may queue if bandwidth is exhausted
/// # struct MyProcess;
/// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
/// # impl dscale::ProcessHandle for MyProcess {
/// #     fn start(&mut self) {}
/// #     fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {}
/// #     fn on_timer(&mut self, id: dscale::TimerId) {}
/// # }
/// ```
///
/// ## Realistic Network Modeling
///
/// ```rust
/// use dscale::{SimulationBuilder, BandwidthDescription};
///
/// // Simulate different network conditions
/// let high_speed = BandwidthDescription::Bounded(1_000_000); // 1MB/jiffy (gigabit-class)
/// let broadband = BandwidthDescription::Bounded(10_000);     // 10KB/jiffy (broadband-class)
/// let mobile = BandwidthDescription::Bounded(1_000);        // 1KB/jiffy (mobile-class)
///
/// let simulation = SimulationBuilder::default()
///     .add_pool::<MyProcess>("datacenter", 10)
///     .nic_bandwidth(broadband) // Realistic home/office bandwidth
///     .build();
/// # struct MyProcess;
/// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
/// # impl dscale::ProcessHandle for MyProcess {
/// #     fn start(&mut self) {}
/// #     fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {}
/// #     fn on_timer(&mut self, id: dscale::TimerId) {}
/// # }
/// ```
///
/// # Impact on System Behavior
///
/// Bandwidth limitations can significantly affect distributed system behavior:
/// - **Backpressure**: Slow receivers can cause senders to queue messages
/// - **Congestion**: Multiple large messages can create transmission delays
/// - **Fairness**: Small messages may be delayed behind large ones
/// - **Timeout Behavior**: Network delays may trigger timeout mechanisms
///
/// # Performance Considerations
///
/// - Unbounded bandwidth has minimal simulation overhead
/// - Bounded bandwidth requires additional computation for queuing and scheduling
/// - Very tight bandwidth limits may create large event queues
/// - Consider the balance between realism and simulation performance
///
/// [`Message::virtual_size`]: crate::Message::virtual_size
/// [`Jiffy`]: crate::Jiffies
#[derive(Clone, Copy)]
pub enum BandwidthDescription {
    /// No bandwidth limitations - messages transmit instantly.
    ///
    /// With unbounded bandwidth, messages are subject only to network latency
    /// (as configured by [`LatencyDescription`]) and are not delayed by
    /// transmission time. This is the most performant option and suitable
    /// for simulations where network bandwidth is not a limiting factor.
    ///
    /// # Use Cases
    ///
    /// - High-speed local networks (datacenter environments)
    /// - Simulations focusing on latency rather than bandwidth
    /// - Performance-critical simulations where bandwidth modeling overhead is unwanted
    /// - Systems where message sizes are small relative to available bandwidth
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{SimulationBuilder, BandwidthDescription};
    ///
    /// let simulation = SimulationBuilder::default()
    ///     .nic_bandwidth(BandwidthDescription::Unbounded)
    ///     .build();
    /// ```
    ///
    /// [`LatencyDescription`]: crate::LatencyDescription
    Unbounded,

    /// Limited bandwidth with specified bytes per jiffy capacity.
    ///
    /// This variant models realistic network bandwidth constraints where
    /// the network interface can transmit at most the specified number
    /// of bytes per jiffy. Messages larger than this limit will take
    /// multiple jiffies to transmit, and multiple messages may queue
    /// if they exceed the available bandwidth.
    ///
    /// # Bandwidth Budget
    ///
    /// Each process maintains a bandwidth budget that:
    /// - Increases by the specified amount each jiffy
    /// - Is consumed by outgoing message transmission
    /// - When exhausted, causes messages to be delayed
    /// - Never exceeds the per-jiffy limit (no "burst" capacity)
    ///
    /// # Parameters
    ///
    /// * `usize` - The maximum number of bytes that can be transmitted per jiffy
    ///
    /// # Transmission Time Calculation
    ///
    /// For a message with virtual size `S` bytes and bandwidth limit `B` bytes/jiffy:
    /// - If `S ≤ B`: Message transmits in 1 jiffy (minimum)
    /// - If `S > B`: Message transmits in `⌈S/B⌉` jiffies
    /// - Multiple messages may extend transmission time further
    ///
    /// # Examples
    ///
    /// ## Basic Bandwidth Limiting
    /// ```rust
    /// use dscale::{SimulationBuilder, BandwidthDescription};
    ///
    /// let simulation = SimulationBuilder::default()
    ///     .nic_bandwidth(BandwidthDescription::Bounded(1000)) // 1KB per jiffy
    ///     .build();
    /// ```
    ///
    /// ## Different Network Classes
    /// ```rust
    /// use dscale::BandwidthDescription;
    ///
    /// // Simulate various network conditions
    /// let fiber = BandwidthDescription::Bounded(100_000);    // 100KB/jiffy
    /// let ethernet = BandwidthDescription::Bounded(10_000);  // 10KB/jiffy
    /// let wifi = BandwidthDescription::Bounded(5_000);       // 5KB/jiffy
    /// let mobile = BandwidthDescription::Bounded(1_000);     // 1KB/jiffy
    /// let dialup = BandwidthDescription::Bounded(56);        // 56 bytes/jiffy
    /// ```
    ///
    /// # Message Size Interaction
    ///
    /// The effectiveness of bandwidth limiting depends on message sizes:
    ///
    /// ```rust
    /// use dscale::{Message, BandwidthDescription};
    ///
    /// struct SmallMessage; // Default virtual_size() = 0
    /// struct LargeMessage { data: Vec<u8> }
    ///
    /// impl Message for SmallMessage {} // No transmission delay
    ///
    /// impl Message for LargeMessage {
    ///     fn virtual_size(&self) -> usize {
    ///         self.data.len() // Will be subject to bandwidth limits
    ///     }
    /// }
    ///
    /// // With Bounded(1000):
    /// // - SmallMessage: transmits instantly (0 bytes)
    /// // - LargeMessage with 2500 bytes: takes 3 jiffies (⌈2500/1000⌉)
    /// ```
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

        // Only for bounded bandwidth - unbounded case is handled directly in deliver_from_latency_queue
        let new_total = self.total_pased[message.step.dest] + message.step.message.virtual_size();

        if new_total > now().0 * self.bandwidth {
            message.arrival_time = Jiffies(new_total / self.bandwidth); // > now()
        }

        self.merged_fifo_buffers.push(std::cmp::Reverse(message));
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
        if self.bandwidth == usize::MAX {
            // For unbounded bandwidth, deliver directly from latency queue
            // (Fast-Path)
            let message = self
                .global_queue
                .pop()
                .expect("Global queue should not be empty");
            Some(message)
        } else {
            // For bounded bandwidth, move to buffers first
            self.move_message_from_latency_queue_to_buffers();
            None
        }
    }
}
