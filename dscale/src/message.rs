//! Message types and handling for inter-process communication.
//!
//! This module defines the core message system used for communication between
//! processes in DScale simulations. It provides the `Message` trait that all
//! message types must implement, as well as `MessagePtr` for type-safe message
//! handling and routing infrastructure.

use std::{any::Any, cmp::Reverse, collections::BinaryHeap, rc::Rc};

use crate::{process_handle::ProcessId, time::Jiffies};

/// Core trait for all message types in DScale simulations.
///
/// The `Message` trait must be implemented by all types that will be sent
/// between processes in the simulation. It extends [`Any`] to enable runtime
/// type checking and provides a mechanism for bandwidth simulation through
/// the [`virtual_size`] method.
///
/// # Purpose
///
/// Messages serve as the primary communication mechanism between processes in
/// DScale simulations. They carry data, commands, responses, and notifications
/// that drive the distributed system behavior being simulated.
///
/// # Bandwidth Simulation
///
/// The [`virtual_size`] method allows messages to specify their simulated size
/// in bytes, which is used by the network simulation to calculate transmission
/// delays based on bandwidth constraints. This enables realistic modeling of
/// network bottlenecks without requiring actual large data payloads in memory.
///
/// # Type Safety
///
/// Messages are delivered as [`MessagePtr`] smart pointers that preserve type
/// information through the [`Any`] trait. This allows safe runtime type
/// checking and downcasting when handling messages.
///
/// # Examples
///
/// ## Simple Message Implementation
///
/// ```rust
/// use dscale::Message;
///
/// struct PingMessage {
///     sequence: u32,
///     timestamp: u64,
/// }
///
/// impl Message for PingMessage {
///     fn virtual_size(&self) -> usize {
///         8 // 4 bytes for sequence + 4 bytes for timestamp
///     }
/// }
/// ```
///
/// ## Large Payload Simulation
///
/// ```rust
/// use dscale::Message;
///
/// struct FileTransferMessage {
///     filename: String,
///     // Simulate a large file without storing actual data
/// }
///
/// impl Message for FileTransferMessage {
///     fn virtual_size(&self) -> usize {
///         // Simulate 1MB file transfer
///         1_000_000 + self.filename.len()
///     }
/// }
/// ```
///
/// ## Default Implementation
///
/// ```rust
/// use dscale::Message;
///
/// struct HeartbeatMessage;
///
/// // Uses default virtual_size() of 0 bytes
/// impl Message for HeartbeatMessage {}
/// ```
///
/// # Message Handling
///
/// Messages are received in process implementations through [`ProcessHandle::on_message`]:
///
/// ```rust
/// use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId};
/// use std::rc::Rc;
///
/// struct MyProcess;
///
/// impl ProcessHandle for MyProcess {
///     fn start(&mut self) {}
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         if let Some(ping) = message.try_as::<PingMessage>() {
///             println!("Received ping with sequence: {}", ping.sequence);
///         }
///     }
///
///     fn on_timer(&mut self, id: TimerId) {}
/// }
/// # struct PingMessage { sequence: u32, timestamp: u64 }
/// # impl dscale::Message for PingMessage {
/// #     fn virtual_size(&self) -> usize { 8 }
/// # }
/// ```
///
/// # Implementation Requirements
///
/// - Must implement [`Any`] (automatically derived)
/// - Should implement [`virtual_size`] if bandwidth simulation is important
/// - Consider implementing [`Clone`] if messages need to be copied
///
/// # Performance Considerations
///
/// - Keep message types lightweight since they may be cloned during routing
/// - Use [`virtual_size`] to simulate large payloads rather than storing actual data
/// - Consider message frequency when designing protocols to avoid overwhelming the simulation
///
/// [`virtual_size`]: Message::virtual_size
/// [`MessagePtr`]: MessagePtr
/// [`ProcessHandle::on_message`]: crate::ProcessHandle::on_message
pub trait Message: Any {
    /// Returns the virtual size of this message in bytes for bandwidth simulation.
    ///
    /// This method defines how large the message appears to the network simulation
    /// for bandwidth calculation purposes. It does not need to match the actual
    /// memory footprint of the message struct - it represents the simulated
    /// network payload size.
    ///
    /// The virtual size is used to:
    /// - Calculate transmission delays based on bandwidth constraints
    /// - Simulate network bottlenecks realistically
    /// - Model the behavior of large data transfers
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `0`, meaning the message consumes
    /// no bandwidth and is transmitted instantaneously (subject only to latency).
    /// This is suitable for small control messages.
    ///
    /// # Examples
    ///
    /// ## Control Message (Zero Size)
    /// ```rust
    /// use dscale::Message;
    ///
    /// struct AckMessage;
    ///
    /// impl Message for AckMessage {
    ///     // Uses default implementation - 0 bytes
    /// }
    /// ```
    ///
    /// ## Data Message (Explicit Size)
    /// ```rust
    /// use dscale::Message;
    ///
    /// struct DataMessage {
    ///     payload: Vec<u8>,
    /// }
    ///
    /// impl Message for DataMessage {
    ///     fn virtual_size(&self) -> usize {
    ///         self.payload.len() + 8 // payload + header overhead
    ///     }
    /// }
    /// ```
    ///
    /// ## Simulated Large Message
    /// ```rust
    /// use dscale::Message;
    ///
    /// struct ImageMessage {
    ///     width: u32,
    ///     height: u32,
    ///     // Actual image data not stored to save memory
    /// }
    ///
    /// impl Message for ImageMessage {
    ///     fn virtual_size(&self) -> usize {
    ///         // Simulate uncompressed RGB image
    ///         (self.width * self.height * 3) as usize
    ///     }
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// The virtual size in bytes as a [`usize`]. Should be 0 or positive.
    fn virtual_size(&self) -> usize {
        usize::default()
    }
}

/// A smart pointer for type-safe message handling in DScale simulations.
///
/// `MessagePtr` is a reference-counted smart pointer that wraps message objects
/// and provides type-safe access methods. It allows the simulation engine to
/// pass messages between processes while preserving type information for
/// safe downcasting at the destination.
///
/// # Purpose
///
/// `MessagePtr` serves several important functions:
/// - **Type Erasure**: Allows storage and passing of heterogeneous message types
/// - **Type Safety**: Provides safe runtime type checking and casting
/// - **Memory Management**: Uses reference counting to manage message lifetime
/// - **Zero-Copy**: Messages can be shared between multiple recipients without copying
///
/// # Usage Patterns
///
/// `MessagePtr` is typically used in two contexts:
/// 1. **Receiving Messages**: In [`ProcessHandle::on_message`] implementations
/// 2. **Internal Routing**: By the simulation engine for message delivery
///
/// # Examples
///
/// ## Basic Message Handling
///
/// ```rust
/// use dscale::{MessagePtr, ProcessHandle, ProcessId, TimerId, Message};
/// use std::rc::Rc;
///
/// struct PingMessage { id: u32 }
/// struct PongMessage { id: u32 }
///
/// impl Message for PingMessage {}
/// impl Message for PongMessage {}
///
/// struct EchoProcess;
///
/// impl ProcessHandle for EchoProcess {
///     fn start(&mut self) {}
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         // Safe type checking and extraction
///         if let Some(ping) = message.try_as::<PingMessage>() {
///             println!("Received ping with ID: {}", ping.id);
///         } else if message.is::<PongMessage>() {
///             // Alternative: check type without extracting
///             let pong = message.as_type::<PongMessage>();
///             println!("Received pong with ID: {}", pong.id);
///         }
///     }
///
///     fn on_timer(&mut self, id: TimerId) {}
/// }
/// ```
///
/// ## Pattern Matching on Message Types
///
/// ```rust
/// use dscale::{MessagePtr, Message};
/// use std::rc::Rc;
///
/// struct RequestMessage { data: String }
/// struct ResponseMessage { result: i32 }
/// struct ErrorMessage { code: u32, description: String }
///
/// impl Message for RequestMessage {}
/// impl Message for ResponseMessage {}
/// impl Message for ErrorMessage {}
///
/// fn handle_message(message: MessagePtr) {
///     if let Some(req) = message.try_as::<RequestMessage>() {
///         println!("Processing request: {}", req.data);
///     } else if let Some(resp) = message.try_as::<ResponseMessage>() {
///         println!("Received response: {}", resp.result);
///     } else if let Some(err) = message.try_as::<ErrorMessage>() {
///         eprintln!("Error {}: {}", err.code, err.description);
///     } else {
///         println!("Unknown message type");
///     }
/// }
/// ```
///
/// [`ProcessHandle::on_message`]: crate::ProcessHandle::on_message
pub struct MessagePtr(pub Rc<dyn Message>);

impl MessagePtr {
    /// Attempts to safely cast the message to a specific type.
    ///
    /// This method provides safe runtime type checking and casting for messages.
    /// It returns `Some(Rc<T>)` if the message is of the requested type `T`,
    /// or `None` if the cast fails. This is the recommended way to handle
    /// messages as it cannot panic.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target message type to cast to. Must be `'static`.
    ///
    /// # Returns
    ///
    /// * `Some(Rc<T>)` - If the message is of type `T`
    /// * `None` - If the message is not of type `T`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{MessagePtr, Message};
    /// # use std::rc::Rc;
    ///
    /// struct PingMessage { id: u32 }
    /// struct PongMessage { id: u32 }
    ///
    /// impl Message for PingMessage {}
    /// impl Message for PongMessage {}
    ///
    /// fn handle_message(message: MessagePtr) {
    ///     // Safe casting with pattern matching
    ///     if let Some(ping) = message.try_as::<PingMessage>() {
    ///         println!("Got ping with ID: {}", ping.id);
    ///     } else if let Some(pong) = message.try_as::<PongMessage>() {
    ///         println!("Got pong with ID: {}", pong.id);
    ///     } else {
    ///         println!("Unknown message type");
    ///     }
    /// }
    /// ```
    pub fn try_as<T: 'static>(&self) -> Option<Rc<T>> {
        match (self.0.clone() as Rc<dyn Any>).downcast::<T>() {
            Err(_) => None,
            Ok(m) => Some(m),
        }
    }

    /// Checks if the message is of a specific type without extracting it.
    ///
    /// This method performs a type check without actually casting the message.
    /// It's useful when you only need to know the message type but don't
    /// need to access the message data immediately.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to check against. Must be `'static`.
    ///
    /// # Returns
    ///
    /// `true` if the message is of type `T`, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{MessagePtr, Message};
    /// # use std::rc::Rc;
    ///
    /// struct ImportantMessage;
    /// struct RegularMessage;
    ///
    /// impl Message for ImportantMessage {}
    /// impl Message for RegularMessage {}
    ///
    /// fn prioritize_message(message: &MessagePtr) -> u8 {
    ///     if message.is::<ImportantMessage>() {
    ///         10 // High priority
    ///     } else if message.is::<RegularMessage>() {
    ///         5  // Normal priority
    ///     } else {
    ///         1  // Low priority for unknown types
    ///     }
    /// }
    /// ```
    pub fn is<T: 'static>(&self) -> bool {
        (self.0.clone() as Rc<dyn Any>).is::<T>()
    }

    /// Casts the message to a specific type, panicking if the cast fails.
    ///
    /// This method performs an unchecked cast to the target type. It should
    /// only be used when you are certain of the message type, typically
    /// after a successful [`is`] check. If the cast fails, this method
    /// will panic.
    ///
    /// **Warning**: This method can panic! Use [`try_as`] for safe casting.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target message type to cast to. Must be `'static`.
    ///
    /// # Returns
    ///
    /// `Rc<T>` - The message cast to the target type.
    ///
    /// # Panics
    ///
    /// Panics if the message is not of type `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{MessagePtr, Message};
    /// # use std::rc::Rc;
    ///
    /// struct StatusMessage { code: u32 }
    /// impl Message for StatusMessage {}
    ///
    /// fn handle_status(message: MessagePtr) {
    ///     // Safe: check type first
    ///     if message.is::<StatusMessage>() {
    ///         let status = message.as_type::<StatusMessage>();
    ///         println!("Status code: {}", status.code);
    ///     }
    ///
    ///     // Unsafe: direct cast without checking
    ///     // let status = message.as_type::<StatusMessage>(); // Could panic!
    /// }
    /// ```
    ///
    /// [`is`]: MessagePtr::is
    /// [`try_as`]: MessagePtr::try_as
    pub fn as_type<T: 'static>(self) -> Rc<T> {
        (self.0 as Rc<dyn Any>).downcast::<T>().unwrap()
    }
}

#[derive(Clone)]
pub struct ProcessStep {
    pub(crate) source: ProcessId,
    pub(crate) dest: ProcessId,
    pub(crate) message: Rc<dyn Message>,
}

#[derive(Clone)]
pub struct RoutedMessage {
    pub(crate) arrival_time: Jiffies,
    pub(crate) step: ProcessStep,
}

impl PartialEq for RoutedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.arrival_time.eq(&other.arrival_time)
    }
}

impl Eq for RoutedMessage {}

impl PartialOrd for RoutedMessage {
    fn ge(&self, other: &Self) -> bool {
        self.arrival_time.ge(&other.arrival_time)
    }
    fn le(&self, other: &Self) -> bool {
        self.arrival_time.le(&other.arrival_time)
    }
    fn gt(&self, other: &Self) -> bool {
        self.arrival_time.gt(&other.arrival_time)
    }
    fn lt(&self, other: &Self) -> bool {
        self.arrival_time.lt(&other.arrival_time)
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.arrival_time.partial_cmp(&other.arrival_time)
    }
}

impl Ord for RoutedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.arrival_time.cmp(&other.arrival_time)
    }
}

pub type TimePriorityMessageQueue = BinaryHeap<Reverse<RoutedMessage>>;
