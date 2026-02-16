//! Value combination utilities for gathering multiple responses.
//!
//! This module provides the `Combiner` struct for collecting a fixed number
//! of values before processing them as a group. This is particularly useful
//! for implementing quorum-based algorithms, consensus protocols, and other
//! distributed system patterns that require waiting for multiple responses.

use std::usize;

/// A runtime-configured collector for gathering multiple values.
///
/// `Combiner` is designed for scenarios where you need to collect exactly a specified
/// number of values before proceeding with computation. It's particularly useful for
/// implementing distributed system patterns such as:
///
/// - **Quorum Systems**: Wait for a majority of responses before making decisions
/// - **Consensus Protocols**: Collect votes or acknowledgments from multiple processes
/// - **Redundant Requests**: Gather responses from multiple replicas
/// - **Batch Processing**: Accumulate items until a threshold is reached
///
/// # Design Philosophy
///
/// - **Runtime Size**: The collection size is specified at runtime during construction,
///   providing flexibility while maintaining efficiency
/// - **One-Shot**: Each combiner instance produces exactly one complete collection
/// - **Type Safety**: Generic over the value type `T` with runtime guarantees
/// - **Memory Efficient**: Uses vector with pre-allocated capacity for efficiency
///
/// # Generic Parameters
///
/// - `T`: The type of values to collect. Must implement `Sized`.
///
/// # Examples
///
/// ## Basic Quorum Pattern
///
/// ```rust
/// use dscale::helpers::Combiner;
///
/// // Collect exactly 3 responses for a quorum
/// let mut quorum: Combiner<String> = Combiner::new(3);
///
/// // Add responses one by one
/// assert!(quorum.combine("vote_yes".to_string()).is_none()); // Not ready yet
/// assert!(quorum.combine("vote_yes".to_string()).is_none()); // Still not ready
///
/// // Third response completes the quorum
/// if let Some(votes) = quorum.combine("vote_no".to_string()) {
///     println!("Quorum achieved: {:?}", votes);
///     // Process the complete set of votes
/// }
/// ```
///
/// ## Consensus Implementation
///
/// ```rust
/// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, Message, send_to};
/// use dscale::helpers::{Combiner, debug_process};
/// use std::rc::Rc;
///
/// #[derive(Clone)]
/// struct VoteMessage {
///     proposal_id: u64,
///     vote: bool,
/// }
/// impl Message for VoteMessage {}
///
/// struct ConsensusProcess {
///     proposal_id: u64,
///     vote_collector: Option<Combiner<bool, 3>>,
/// }
///
/// impl ProcessHandle for ConsensusProcess {
///     fn start(&mut self) {
///         // Start a new consensus round
///         self.proposal_id = 1;
///         self.vote_collector = Some(Combiner::new(3));
///
///         // Send vote requests to other processes
///         // send_to(1, VoteMessage { proposal_id: 1, vote: true });
///         // send_to(2, VoteMessage { proposal_id: 1, vote: true });
///         // send_to(3, VoteMessage { proposal_id: 1, vote: false });
///     }
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         if let Some(vote_msg) = message.try_as::<VoteMessage>() {
///             if vote_msg.proposal_id == self.proposal_id {
///                 if let Some(ref mut collector) = self.vote_collector {
///                     if let Some(votes) = collector.combine(vote_msg.vote) {
///                         debug_process!("Collected all votes: {:?}", votes);
///                         let yes_count = votes.iter().filter(|&&v| v).count();
///                         let consensus = yes_count >= 2; // Majority rule
///                         debug_process!("Consensus result: {}", consensus);
///                         self.vote_collector = None; // Reset for next round
///                     }
///                 }
///             }
///         }
///     }
///
///     fn on_timer(&mut self, id: TimerId) {}
/// }
/// # impl Default for ConsensusProcess {
/// #     fn default() -> Self {
/// #         Self { proposal_id: 0, vote_collector: None }
/// #     }
/// # }
/// ```
///
/// ## Response Aggregation
///
/// ```rust
/// use dscale::helpers::Combiner;
///
/// #[derive(Debug)]
/// struct ServerResponse {
///     server_id: u32,
///     latency: u64,
///     data: String,
/// }
///
/// fn collect_responses() {
///     let mut collector: Combiner<ServerResponse> = Combiner::new(5);
///
///     // Simulate receiving responses from 5 servers
///     let responses = vec![
///         ServerResponse { server_id: 1, latency: 10, data: "result1".to_string() },
///         ServerResponse { server_id: 2, latency: 15, data: "result2".to_string() },
///         ServerResponse { server_id: 3, latency: 8,  data: "result3".to_string() },
///         ServerResponse { server_id: 4, latency: 12, data: "result4".to_string() },
///         ServerResponse { server_id: 5, latency: 20, data: "result5".to_string() },
///     ];
///
///     for response in responses {
///         if let Some(all_responses) = collector.combine(response) {
///             // All responses collected - find fastest
///             let fastest = all_responses.iter()
///                 .min_by_key(|r| r.latency)
///                 .unwrap();
///             println!("Fastest response from server {}: {}", fastest.server_id, fastest.data);
///             break;
///         }
///     }
/// }
/// ```
///
/// # Performance Characteristics
///
/// - **Memory**: Uses vector with pre-allocated capacity
/// - **Time Complexity**: O(1) for `combine()` operations
/// - **Space Complexity**: O(threshold) where threshold is the collection size
/// - **Minimal Allocation**: Single allocation during construction
///
/// # Common Use Cases in Distributed Systems
///
/// - **Byzantine Fault Tolerance**: Collect 2f+1 responses in f-fault-tolerant systems
/// - **Read Quorums**: Wait for majority of replicas before returning data
/// - **Write Acknowledgments**: Ensure sufficient replicas confirm writes
/// - **Leader Election**: Collect votes from majority of processes
/// - **Consensus Algorithms**: Gather proposals or votes for Raft, PBFT, etc.
///
/// # Thread Safety
///
/// `Combiner` is not thread-safe by default, but this is not a concern in
/// DScale's single-threaded simulation environment.
pub struct Combiner<T: Sized> {
    values: Vec<T>,
    threshold: usize,
    idx: usize,
}

impl<T: Sized> Combiner<T> {
    /// Creates a new combiner that will collect exactly `threshold` values.
    ///
    /// This constructor initializes an empty combiner ready to accept values
    /// through the [`combine`] method. The combiner will return `None` from
    /// [`combine`] until exactly `threshold` values have been provided.
    ///
    /// # Requirements
    ///
    /// The `threshold` must be greater than 0. This is enforced by a debug
    /// assertion to catch programming errors during development.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::helpers::Combiner;
    ///
    /// // Create combiners for different quorum sizes
    /// let simple_majority: Combiner<bool> = Combiner::new(3);
    /// let byzantine_quorum: Combiner<String> = Combiner::new(7); // 2f+1 for f=3
    /// let unanimous: Combiner<u32> = Combiner::new(5);
    /// ```
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `threshold` is 0.
    ///
    /// [`combine`]: Combiner::combine
    pub fn new(threshold: usize) -> Self {
        debug_assert!(
            threshold > 0,
            "Combinter threshold should be greater than zero"
        );
        Self {
            values: Vec::with_capacity(threshold),
            threshold,
            idx: 0,
        }
    }

    /// Adds a value to the combiner and returns the complete collection when ready.
    ///
    /// This method accepts one value and adds it to the internal collection.
    /// It returns:
    /// - `None` if fewer than `threshold` values have been collected
    /// - `Some(&[T])` when exactly `threshold` values have been collected
    ///
    /// Once a complete collection is returned, the combiner is considered
    /// "consumed" and subsequent calls will always return `None`.
    ///
    /// # Behavior
    ///
    /// - **Before Completion**: Returns `None` and stores the value internally
    /// - **At Completion**: Returns `Some(slice)` containing all values in order
    /// - **After Completion**: Always returns `None` (combiner is exhausted)
    ///
    /// # Parameters
    ///
    /// * `value` - A value of type `T` to add to the collection
    ///
    /// # Returns
    ///
    /// - `None` if the collection is not yet complete
    /// - `Some(&[T])` when the collection is complete, containing all values in insertion order
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// ```rust
    /// use dscale::helpers::Combiner;
    ///
    /// let mut collector: Combiner<i32> = Combiner::new(3);
    ///
    /// // First two values return None
    /// assert!(collector.combine(10).is_none());
    /// assert!(collector.combine(20).is_none());
    ///
    /// // Third value completes the collection
    /// if let Some(values) = collector.combine(30) {
    ///     assert_eq!(values, &[10, 20, 30]);
    /// }
    ///
    /// // Subsequent calls return None
    /// assert!(collector.combine(40).is_none());
    /// ```
    ///
    /// ## Quorum Voting Example
    ///
    /// ```rust
    /// use dscale::helpers::Combiner;
    ///
    /// fn process_votes() -> bool {
    ///     let mut vote_collector: Combiner<bool> = Combiner::new(5);
    ///
    ///     // Simulate receiving votes
    ///     let votes = [true, true, false, true, false];
    ///
    ///     for vote in votes {
    ///         if let Some(all_votes) = vote_collector.combine(vote) {
    ///             // Count yes votes
    ///             let yes_votes = all_votes.iter().filter(|&&v| v).count();
    ///             return yes_votes > all_votes.len() / 2; // Majority rule
    ///         }
    ///     }
    ///
    ///     false // Shouldn't reach here in this example
    /// }
    /// ```
    ///
    /// ## Error Handling Pattern
    ///
    /// ```rust
    /// use dscale::helpers::Combiner;
    ///
    /// #[derive(Debug)]
    /// enum Response {
    ///     Success(String),
    ///     Error(u32),
    /// }
    ///
    /// fn handle_responses() {
    ///     let mut collector: Combiner<Response> = Combiner::new(3);
    ///
    ///     // Process responses as they arrive
    ///     let responses = [
    ///         Response::Success("OK".to_string()),
    ///         Response::Error(404),
    ///         Response::Success("Done".to_string()),
    ///     ];
    ///
    ///     for response in responses {
    ///         if let Some(all_responses) = collector.combine(response) {
    ///             let errors: Vec<_> = all_responses.iter()
    ///                 .filter_map(|r| match r {
    ///                     Response::Error(code) => Some(code),
    ///                     _ => None,
    ///                 })
    ///                 .collect();
    ///
    ///             if !errors.is_empty() {
    ///                 println!("Received errors: {:?}", errors);
    ///             } else {
    ///                 println!("All responses successful");
    ///             }
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// - Values are stored in insertion order
    /// - Memory is pre-allocated on the stack for efficiency
    /// - The operation is O(1) with no heap allocations
    pub fn combine(&mut self, value: T) -> Option<&[T]> {
        if self.idx >= self.threshold {
            return None;
        }

        self.values.push(value);
        self.idx += 1;

        if self.idx == self.threshold {
            Some(&self.values)
        } else {
            None
        }
    }
}
