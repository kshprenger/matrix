//! Process trait and identification types for DScale simulations.
//!
//! This module defines the core `ProcessHandle` trait that must be implemented
//! by all processes in DScale simulations, as well as the `ProcessId` type used
//! for process identification throughout the system.

use std::{cell::RefCell, rc::Rc};

use crate::{MessagePtr, time::timer_manager::TimerId};

/// Unique identifier for a process within a simulation.
///
/// `ProcessId` is a numeric identifier that uniquely identifies each process
/// within a single simulation run. Process IDs are assigned automatically
/// by the simulation engine when processes are created through
/// [`SimulationBuilder::add_pool`].
///
/// Process IDs are used for:
/// - Message routing with [`send_to`]
/// - Identifying message senders in [`ProcessHandle::on_message`]
/// - Process identification in logging and debugging
/// - Pool membership queries
///
/// # Properties
///
/// - **Unique**: Each process has a distinct ID within a simulation
/// - **Stable**: IDs don't change during simulation execution
/// - **Sequential**: IDs are assigned in order of process creation
/// - **Deterministic**: Same configuration produces same ID assignments
///
/// # Examples
///
/// ```rust
/// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, rank, send_to};
/// use dscale::helpers::debug_process;
///
/// struct MyProcess;
///
/// impl ProcessHandle for MyProcess {
///     fn start(&mut self) {
///         let my_id: ProcessId = rank();
///         debug_process!("My process ID is: {}", my_id);
///     }
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         debug_process!("Received message from process {}", from);
///
///         // Echo back to sender
///         // send_to(from, SomeMessage);
///     }
///
///     fn on_timer(&mut self, id: TimerId) {}
/// }
/// ```
///
/// [`SimulationBuilder::add_pool`]: crate::SimulationBuilder::add_pool
/// [`send_to`]: crate::send_to
/// [`ProcessHandle::on_message`]: ProcessHandle::on_message
pub type ProcessId = usize;

pub(crate) type MutableProcessHandle = Rc<RefCell<dyn ProcessHandle>>;

/// Core trait that defines the behavior of a process in DScale simulations.
///
/// `ProcessHandle` is the fundamental interface that all processes must implement
/// to participate in a DScale simulation. It defines three key lifecycle methods
/// that the simulation engine calls to drive process behavior:
///
/// - [`start`]: Initialize the process and schedule initial work
/// - [`on_message`]: React to incoming messages from other processes
/// - [`on_timer`]: Handle timer events scheduled by the process
///
/// # Implementation Requirements
///
/// Processes must also implement:
/// - [`Default`]: For automatic instantiation by the simulation builder
/// - `'static`: To ensure the process can be stored in the simulation
///
/// # Process Lifecycle
///
/// 1. **Creation**: Processes are created using their [`Default`] implementation
/// 2. **Initialization**: [`start`] is called once to begin process execution
/// 3. **Event Loop**: [`on_message`] and [`on_timer`] are called as events occur
/// 4. **Termination**: Process ends when simulation completes
///
/// # Context and Global Functions
///
/// Within the context of `ProcessHandle` methods, you have access to global
/// simulation functions:
///
/// - **Communication**: [`send_to`], [`broadcast`], [`broadcast_within_pool`]
/// - **Timing**: [`schedule_timer_after`], [`now`]
/// - **Identity**: [`rank`] (current process ID)
/// - **Topology**: [`list_pool`], [`choose_from_pool`]
/// - **Utilities**: [`global_unique_id`]
///
/// # Examples
///
/// ## Basic Process Implementation
///
/// ```rust
/// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, Message};
/// use dscale::{send_to, schedule_timer_after, broadcast, rank, Jiffies};
/// use dscale::helpers::debug_process;
/// use std::rc::Rc;
///
/// // Define a message type
/// struct PingMessage {
///     sequence: u32,
/// }
///
/// impl Message for PingMessage {
///     fn virtual_size(&self) -> usize {
///         4 // 4 bytes for u32
///     }
/// }
///
/// // Process implementation
/// #[derive(Default)]
/// struct PingPongProcess {
///     sequence: u32,
/// }
///
/// impl ProcessHandle for PingPongProcess {
///     fn start(&mut self) {
///         debug_process!("Starting PingPong process");
///
///         // Schedule initial work - send a ping message to ourselves
///         let my_id = rank();
///         send_to(my_id, PingMessage { sequence: 0 });
///
///         // Also schedule a periodic timer
///         schedule_timer_after(Jiffies(1000));
///     }
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         if let Some(ping) = message.try_as::<PingMessage>() {
///             debug_process!("Received ping {} from {}", ping.sequence, from);
///
///             // Respond with next sequence number
///             self.sequence += 1;
///             send_to(from, PingMessage { sequence: self.sequence });
///         }
///     }
///
///     fn on_timer(&mut self, _id: TimerId) {
///         debug_process!("Timer fired, broadcasting status");
///         broadcast(PingMessage { sequence: self.sequence });
///
///         // Reschedule timer
///         schedule_timer_after(Jiffies(1000));
///     }
/// }
/// ```
///
/// ## State Management Pattern
///
/// ```rust
/// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, schedule_timer_after, Jiffies};
/// use dscale::global::anykv;
///
/// #[derive(Default)]
/// struct StatefulProcess {
///     state: ProcessState,
/// }
///
/// #[derive(Default)]
/// enum ProcessState {
///     #[default]
///     Initializing,
///     Active,
///     Shutting Down,
/// }
///
/// impl ProcessHandle for StatefulProcess {
///     fn start(&mut self) {
///         self.state = ProcessState::Initializing;
///
///         // Initialize state in global storage
///         anykv::set("process_count", 0u32);
///
///         // Transition to active after delay
///         schedule_timer_after(Jiffies(100));
///     }
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         match self.state {
///             ProcessState::Active => {
///                 // Handle messages normally
///             }
///             _ => {
///                 // Ignore messages in other states
///             }
///         }
///     }
///
///     fn on_timer(&mut self, _id: TimerId) {
///         match self.state {
///             ProcessState::Initializing => {
///                 self.state = ProcessState::Active;
///                 anykv::modify("process_count", |count: &mut u32| *count += 1);
///             }
///             ProcessState::Active => {
///                 // Handle periodic work
///             }
///             ProcessState::ShuttingDown => {
///                 // Cleanup
///             }
///         }
///     }
/// }
/// ```
///
/// # Design Patterns
///
/// ## Event-Driven Architecture
/// Processes should be designed around reacting to events (messages and timers)
/// rather than continuous execution loops.
///
/// ## State Machines
/// Complex processes often benefit from explicit state machine patterns
/// where behavior changes based on current state.
///
/// ## Message Passing
/// Communication between processes should happen exclusively through
/// message passing using the provided global functions.
///
/// ## Timer-Based Coordination
/// Use timers for periodic work, timeouts, and scheduling future actions.
///
/// [`start`]: ProcessHandle::start
/// [`on_message`]: ProcessHandle::on_message
/// [`on_timer`]: ProcessHandle::on_timer
/// [`send_to`]: crate::send_to
/// [`broadcast`]: crate::broadcast
/// [`broadcast_within_pool`]: crate::broadcast_within_pool
/// [`schedule_timer_after`]: crate::schedule_timer_after
/// [`now`]: crate::now
/// [`rank`]: crate::rank
/// [`list_pool`]: crate::list_pool
/// [`choose_from_pool`]: crate::choose_from_pool
/// [`global_unique_id`]: crate::global_unique_id
pub trait ProcessHandle {
    /// Initialize the process and schedule initial work.
    ///
    /// This method is called exactly once for each process at the beginning
    /// of the simulation, after all processes have been created but before
    /// any message processing begins. It's the entry point for process logic
    /// and should set up the initial state and schedule any initial work.
    ///
    /// # Purpose
    ///
    /// The `start` method serves to:
    /// - Initialize process-specific state
    /// - Schedule initial messages or timers
    /// - Register with other processes if needed
    /// - Set up recurring work patterns
    ///
    /// # Context
    ///
    /// During `start` execution, all global simulation functions are available:
    /// - Send messages to other processes
    /// - Schedule timers for future execution
    /// - Access process identity and topology information
    /// - Initialize shared state in the global key-value store
    ///
    /// # Requirements
    ///
    /// The implementation **must** schedule some form of continuing work,
    /// either by:
    /// - Sending messages (which will trigger `on_message` calls)
    /// - Scheduling timers (which will trigger `on_timer` calls)
    /// - Both
    ///
    /// Failure to schedule continuing work may result in process deadlock
    /// where the process has no more events to process.
    ///
    /// # Examples
    ///
    /// ## Simple Initialization
    /// ```rust
    /// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId};
    /// use dscale::{schedule_timer_after, rank, Jiffies};
    /// use dscale::helpers::debug_process;
    ///
    /// #[derive(Default)]
    /// struct SimpleProcess;
    ///
    /// impl ProcessHandle for SimpleProcess {
    ///     fn start(&mut self) {
    ///         let my_id = rank();
    ///         debug_process!("Process {} starting up", my_id);
    ///
    ///         // Schedule initial timer
    ///         schedule_timer_after(Jiffies(100));
    ///     }
    ///
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {}
    ///     fn on_timer(&mut self, id: TimerId) {
    ///         debug_process!("Timer fired - doing work");
    ///         // Schedule next timer for continuing work
    ///         schedule_timer_after(Jiffies(1000));
    ///     }
    /// }
    /// ```
    ///
    /// ## Client-Server Initialization
    /// ```rust
    /// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, Message};
    /// use dscale::{list_pool, send_to, choose_from_pool};
    /// use std::rc::Rc;
    ///
    /// struct RequestMessage;
    /// impl Message for RequestMessage {}
    ///
    /// #[derive(Default)]
    /// struct ClientProcess;
    ///
    /// impl ProcessHandle for ClientProcess {
    ///     fn start(&mut self) {
    ///         // Find a server to connect to
    ///         let server_id = choose_from_pool("servers");
    ///
    ///         // Send initial request
    ///         send_to(server_id, RequestMessage);
    ///     }
    ///
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {}
    ///     fn on_timer(&mut self, id: TimerId) {}
    /// }
    /// ```
    fn start(&mut self);

    /// Handle an incoming message from another process.
    ///
    /// This method is called whenever the current process receives a message
    /// sent by another process using functions like [`send_to`], [`broadcast`],
    /// or [`broadcast_within_pool`]. It's the primary mechanism for inter-process
    /// communication in DScale simulations.
    ///
    /// # Parameters
    ///
    /// * `from` - The [`ProcessId`] of the process that sent the message
    /// * `message` - A [`MessagePtr`] containing the message data
    ///
    /// # Message Handling
    ///
    /// Messages are delivered as [`MessagePtr`] smart pointers that can contain
    /// any type implementing the [`Message`] trait. Use the provided methods
    /// to inspect and extract message data:
    ///
    /// - [`MessagePtr::try_as`] - Safely attempt to cast to specific type
    /// - [`MessagePtr::is`] - Check if message is of specific type
    /// - [`MessagePtr::as_type`] - Cast to specific type (panics if wrong)
    ///
    /// # Timing and Ordering
    ///
    /// - Messages are delivered in the order determined by network latency simulation
    /// - The simulation clock advances to the message delivery time before calling this method
    /// - Multiple messages may be delivered to the same process at the same simulation time
    ///
    /// # Examples
    ///
    /// ## Basic Message Handling
    /// ```rust
    /// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, Message};
    /// use dscale::helpers::debug_process;
    /// use std::rc::Rc;
    ///
    /// struct PingMessage { id: u32 }
    /// struct PongMessage { id: u32 }
    ///
    /// impl Message for PingMessage {}
    /// impl Message for PongMessage {}
    ///
    /// #[derive(Default)]
    /// struct EchoProcess;
    ///
    /// impl ProcessHandle for EchoProcess {
    ///     fn start(&mut self) {}
    ///
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
    ///         if let Some(ping) = message.try_as::<PingMessage>() {
    ///             debug_process!("Received ping {} from {}", ping.id, from);
    ///             // Echo back with pong
    ///             // send_to(from, PongMessage { id: ping.id });
    ///         } else if let Some(pong) = message.try_as::<PongMessage>() {
    ///             debug_process!("Received pong {} from {}", pong.id, from);
    ///         } else {
    ///             debug_process!("Received unknown message from {}", from);
    ///         }
    ///     }
    ///
    ///     fn on_timer(&mut self, id: TimerId) {}
    /// }
    /// ```
    ///
    /// ## State-Based Message Handling
    /// ```rust
    /// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, Message};
    ///
    /// struct JoinMessage;
    /// struct DataMessage { payload: Vec<u8> }
    /// impl Message for JoinMessage {}
    /// impl Message for DataMessage {}
    ///
    /// #[derive(Default)]
    /// struct StatefulProcess {
    ///     joined: bool,
    ///     peers: Vec<ProcessId>,
    /// }
    ///
    /// impl ProcessHandle for StatefulProcess {
    ///     fn start(&mut self) {}
    ///
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
    ///         if let Some(_join) = message.try_as::<JoinMessage>() {
    ///             if !self.peers.contains(&from) {
    ///                 self.peers.push(from);
    ///             }
    ///         } else if let Some(data) = message.try_as::<DataMessage>() {
    ///             if self.joined {
    ///                 // Process data only if we're in joined state
    ///                 self.handle_data(&data.payload);
    ///             }
    ///         }
    ///     }
    ///
    ///     fn on_timer(&mut self, id: TimerId) {}
    /// }
    ///
    /// impl StatefulProcess {
    ///     fn handle_data(&mut self, payload: &[u8]) {
    ///         // Process the data payload
    ///     }
    /// }
    /// ```
    ///
    /// # Best Practices
    ///
    /// - **Pattern Match**: Use pattern matching on message types for clean code
    /// - **State Awareness**: Consider current process state when handling messages
    /// - **Error Handling**: Gracefully handle unexpected or malformed messages
    /// - **Response Patterns**: Establish clear request-response patterns where appropriate
    /// - **Avoid Blocking**: Don't perform long computations; break work into smaller pieces
    ///
    /// [`send_to`]: crate::send_to
    /// [`broadcast`]: crate::broadcast
    /// [`broadcast_within_pool`]: crate::broadcast_within_pool
    /// [`MessagePtr`]: crate::MessagePtr
    /// [`MessagePtr::try_as`]: crate::MessagePtr::try_as
    /// [`MessagePtr::is`]: crate::MessagePtr::is
    /// [`MessagePtr::as_type`]: crate::MessagePtr::as_type
    /// [`Message`]: crate::Message
    fn on_message(&mut self, from: ProcessId, message: MessagePtr);

    /// Handle a timer event scheduled by this process.
    ///
    /// This method is called when a timer scheduled using [`schedule_timer_after`]
    /// reaches its scheduled time. Timers are useful for implementing timeouts,
    /// periodic work, delayed actions, and state machine transitions.
    ///
    /// # Parameters
    ///
    /// * `id` - The [`TimerId`] that was returned when the timer was scheduled
    ///
    /// # Timer Management
    ///
    /// - **Identification**: Use the timer ID to distinguish between different timers
    /// - **One-Shot**: Each timer fires exactly once and is then removed
    /// - **Rescheduling**: Create recurring behavior by scheduling new timers
    /// - **Cancellation**: No built-in cancellation; implement cancellation logic in your process
    ///
    /// # Timing Guarantees
    ///
    /// - Timers fire at exactly the scheduled simulation time
    /// - The simulation clock is advanced to the timer's time before calling this method
    /// - Multiple timers may fire at the same simulation time
    ///
    /// # Examples
    ///
    /// ## Basic Timer Handling
    /// ```rust
    /// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId};
    /// use dscale::{schedule_timer_after, Jiffies, now};
    /// use dscale::helpers::debug_process;
    ///
    /// #[derive(Default)]
    /// struct TimerProcess {
    ///     heartbeat_timer: Option<TimerId>,
    /// }
    ///
    /// impl ProcessHandle for TimerProcess {
    ///     fn start(&mut self) {
    ///         // Schedule initial heartbeat
    ///         self.heartbeat_timer = Some(schedule_timer_after(Jiffies(1000)));
    ///     }
    ///
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {}
    ///
    ///     fn on_timer(&mut self, id: TimerId) {
    ///         if Some(id) == self.heartbeat_timer {
    ///             debug_process!("Heartbeat at time {}", now());
    ///
    ///             // Reschedule for next heartbeat
    ///             self.heartbeat_timer = Some(schedule_timer_after(Jiffies(1000)));
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// ## Multiple Timer Types
    /// ```rust
    /// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId};
    /// use dscale::{schedule_timer_after, Jiffies};
    /// use dscale::helpers::debug_process;
    ///
    /// #[derive(Default)]
    /// struct MultiTimerProcess {
    ///     heartbeat_timer: Option<TimerId>,
    ///     timeout_timer: Option<TimerId>,
    ///     cleanup_timer: Option<TimerId>,
    /// }
    ///
    /// impl ProcessHandle for MultiTimerProcess {
    ///     fn start(&mut self) {
    ///         self.heartbeat_timer = Some(schedule_timer_after(Jiffies(1000)));
    ///         self.timeout_timer = Some(schedule_timer_after(Jiffies(5000)));
    ///         self.cleanup_timer = Some(schedule_timer_after(Jiffies(60000)));
    ///     }
    ///
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
    ///         // Reset timeout on any message
    ///         self.timeout_timer = Some(schedule_timer_after(Jiffies(5000)));
    ///     }
    ///
    ///     fn on_timer(&mut self, id: TimerId) {
    ///         if Some(id) == self.heartbeat_timer {
    ///             debug_process!("Sending heartbeat");
    ///             self.heartbeat_timer = Some(schedule_timer_after(Jiffies(1000)));
    ///         } else if Some(id) == self.timeout_timer {
    ///             debug_process!("Timeout occurred!");
    ///             self.timeout_timer = None; // Don't reschedule
    ///         } else if Some(id) == self.cleanup_timer {
    ///             debug_process!("Performing cleanup");
    ///             self.cleanup_timer = Some(schedule_timer_after(Jiffies(60000)));
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// ## State Machine with Timers
    /// ```rust
    /// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId};
    /// use dscale::{schedule_timer_after, Jiffies};
    ///
    /// #[derive(Default)]
    /// struct StateMachineProcess {
    ///     state: State,
    ///     transition_timer: Option<TimerId>,
    /// }
    ///
    /// #[derive(Default)]
    /// enum State {
    ///     #[default]
    ///     Initializing,
    ///     Active,
    ///     Cooldown,
    /// }
    ///
    /// impl ProcessHandle for StateMachineProcess {
    ///     fn start(&mut self) {
    ///         // Transition to active after initialization delay
    ///         self.transition_timer = Some(schedule_timer_after(Jiffies(100)));
    ///     }
    ///
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {}
    ///
    ///     fn on_timer(&mut self, id: TimerId) {
    ///         if Some(id) == self.transition_timer {
    ///             match self.state {
    ///                 State::Initializing => {
    ///                     self.state = State::Active;
    ///                     // Schedule transition to cooldown
    ///                     self.transition_timer = Some(schedule_timer_after(Jiffies(5000)));
    ///                 }
    ///                 State::Active => {
    ///                     self.state = State::Cooldown;
    ///                     // Schedule return to active
    ///                     self.transition_timer = Some(schedule_timer_after(Jiffies(2000)));
    ///                 }
    ///                 State::Cooldown => {
    ///                     self.state = State::Active;
    ///                     // Schedule next cooldown
    ///                     self.transition_timer = Some(schedule_timer_after(Jiffies(5000)));
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Common Use Cases
    ///
    /// - **Heartbeats**: Regular status messages or keep-alive signals
    /// - **Timeouts**: Detect failed operations or unresponsive peers
    /// - **Periodic Work**: Regular maintenance, metrics collection, or state synchronization
    /// - **Delayed Actions**: Implement exponential backoff or scheduled operations
    /// - **State Transitions**: Drive state machine progressions
    ///
    /// [`schedule_timer_after`]: crate::schedule_timer_after
    /// [`TimerId`]: crate::TimerId
    fn on_timer(&mut self, id: TimerId);
}
