use crate::{MessagePtr, ProcessId, process::Configuration};

pub trait ProcessHandle {
    /// This methods provides initial configuration to the process.
    /// It is also requires process to schedule some initial messages.
    fn Bootstrap(&mut self, configuration: Configuration);

    /// Deliver message with source process
    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr);
}
