//! Global unique identifier generation.
//!
//! This module provides a thread-safe, globally unique identifier generator
//! that can be used throughout the simulation. The TSO (Timestamp Oracle)
//! generates monotonically increasing unique identifiers within a single
//! simulation run.
//!
//! The unique IDs are useful for creating identifiers for messages, timers,
//! or any other simulation entities that need globally unique identification.

use std::cell::Cell;

thread_local! {
    pub(crate) static TSO: Cell<usize> = Cell::new(0)
}

/// Generates a globally unique identifier within the simulation.
///
/// This function returns a monotonically increasing unique identifier that
/// is guaranteed to be unique within the current simulation run. Each call
/// returns a different value, making it suitable for creating unique IDs
/// for timers, messages, or other simulation entities.
///
/// The identifier is generated using a thread-local counter that increments
/// with each call, ensuring both uniqueness and deterministic behavior across
/// simulation runs with the same configuration.
///
/// # Context
///
/// This function can be called from any context within the simulation,
/// including from within [`ProcessHandle`] implementations, timer callbacks,
/// and message handlers.
///
/// [`ProcessHandle`]: crate::ProcessHandle
///
/// # Examples
///
/// ```rust
/// use dscale::{global_unique_id, ProcessHandle, ProcessId, MessagePtr, TimerId};
/// use dscale::helpers::debug_process;
///
/// struct MyProcess {
///     session_id: usize,
/// }
///
/// impl Default for MyProcess {
///     fn default() -> Self {
///         Self {
///             session_id: global_unique_id(),
///         }
///     }
/// }
///
/// impl ProcessHandle for MyProcess {
///     fn start(&mut self) {
///         debug_process!("Starting with session ID: {}", self.session_id);
///
///         // Generate unique IDs for different purposes
///         let request_id = global_unique_id();
///         let transaction_id = global_unique_id();
///
///         debug_process!("Request ID: {}, Transaction ID: {}", request_id, transaction_id);
///     }
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         let message_id = global_unique_id();
///         debug_process!("Processing message with ID: {}", message_id);
///     }
///
///     fn on_timer(&mut self, id: TimerId) {}
/// }
/// ```
///
/// # Returns
///
/// A unique `usize` identifier that has never been returned before in the
/// current simulation run.
///
/// # Thread Safety
///
pub fn global_unique_id() -> usize {
    TSO.replace(TSO.get() + 1)
}

pub(crate) fn drop_tso() {
    TSO.take();
}
