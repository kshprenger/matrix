//! Timer management for process scheduling in DScale simulations.
//!
//! This module provides timer functionality that allows processes to schedule
//! delayed execution of callbacks. Timers are managed centrally by the simulation
//! engine and fire deterministically based on simulation time progression.

use std::{cell::RefCell, cmp::Reverse, collections::BinaryHeap, rc::Rc};

use log::debug;

use crate::{
    ProcessId,
    actor::{EventSubmitter, SimulationActor},
    dscale_message::DScaleMessage,
    global, now,
    nursery::Nursery,
    time::Jiffies,
};

/// Unique identifier for scheduled timers.
///
/// `TimerId` is a unique identifier returned when scheduling a timer using
/// [`schedule_timer_after`]. This identifier is passed back to the process
/// when the timer fires, allowing the process to distinguish between
/// different active timers.
///
/// Timer IDs are guaranteed to be unique within a single simulation run
/// and are generated using the global unique ID system.
///
/// # Usage
///
/// Timer IDs are primarily used in two contexts:
/// 1. **Scheduling**: Returned by [`schedule_timer_after`] to identify the timer
/// 2. **Handling**: Passed to [`ProcessHandle::on_timer`] when the timer fires
///
/// # Examples
///
/// ```rust
/// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, schedule_timer_after, Jiffies};
/// use dscale::helpers::debug_process;
///
/// struct MyProcess {
///     heartbeat_timer: Option<TimerId>,
///     timeout_timer: Option<TimerId>,
/// }
///
/// impl ProcessHandle for MyProcess {
///     fn start(&mut self) {
///         // Schedule a recurring heartbeat
///         self.heartbeat_timer = Some(schedule_timer_after(Jiffies(1000)));
///
///         // Schedule a timeout
///         self.timeout_timer = Some(schedule_timer_after(Jiffies(5000)));
///     }
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         // Cancel timeout on message receipt
///         self.timeout_timer = None;
///     }
///
///     fn on_timer(&mut self, id: TimerId) {
///         if Some(id) == self.heartbeat_timer {
///             debug_process!("Heartbeat timer fired");
///             // Reschedule heartbeat
///             self.heartbeat_timer = Some(schedule_timer_after(Jiffies(1000)));
///         } else if Some(id) == self.timeout_timer {
///             debug_process!("Timeout occurred");
///             self.timeout_timer = None;
///         }
///     }
/// }
/// ```
///
/// # Implementation Details
///
/// - Timer IDs are implemented as `usize` values
/// - IDs are generated using [`global_unique_id`] to ensure uniqueness
/// - Timer IDs are only valid within the simulation run that created them
/// - There is no built-in timer cancellation mechanism (implement cancellation logic in your process)
///
/// [`schedule_timer_after`]: crate::schedule_timer_after
/// [`ProcessHandle::on_timer`]: crate::ProcessHandle::on_timer
/// [`global_unique_id`]: crate::global_unique_id
pub type TimerId = usize;

pub(crate) fn next_timer_id() -> TimerId {
    global::global_unique_id()
}

pub(crate) type TimerManagerActor = Rc<RefCell<TimerManager>>;

pub(crate) struct TimerManager {
    working_timers: BinaryHeap<Reverse<(Jiffies, (ProcessId, TimerId))>>,
    nursery: Rc<Nursery>,
}

impl TimerManager {
    pub(crate) fn new(nursery: Rc<Nursery>) -> Self {
        Self {
            working_timers: BinaryHeap::new(),
            nursery,
        }
    }
}

impl SimulationActor for TimerManager {
    fn start(&mut self) {
        // Do nothing
    }

    fn peek_closest(&self) -> Option<Jiffies> {
        self.working_timers.peek().map(|entry| entry.0.0)
    }

    fn step(&mut self) {
        let (_, (process_id, timer_id)) = self.working_timers.pop().expect("Should not be empty").0;
        debug!("Firing timer with TimerId {timer_id} for P{process_id}");
        self.nursery
            .deliver(process_id, process_id, DScaleMessage::Timer(timer_id));
    }
}

impl EventSubmitter for TimerManager {
    type Event = (ProcessId, TimerId, Jiffies);

    fn submit(&mut self, events: &mut Vec<Self::Event>) {
        events.drain(..).for_each(|(source, timer_id, after)| {
            self.working_timers
                .push(Reverse((now() + after, (source, timer_id))));
        });
    }
}
