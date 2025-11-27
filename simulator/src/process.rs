use crate::{Message, OutgoingMessages};

pub type ProcessId = usize;

pub trait ProcessHandle<M: Message> {
    /// This methods provides initial configuration to the process. That currently includes only assigned ProcessId.
    /// It is also requires process to schedule some initial events.
    fn bootstrap(&mut self, assigned_id: ProcessId, outgoing: &mut OutgoingMessages<M>);

    /// Deliver event with source process
    fn on_message(&mut self, from: ProcessId, message: M, outgoing: &mut OutgoingMessages<M>);
}
