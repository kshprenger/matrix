use crate::{Destination, ProcessId, communication::Message};

/// Injected into user's process's on_message handler.
/// This is common interface to process to allow it schedule some events.
pub struct OutgoingMessages<M: Message>(pub(crate) Vec<(Destination, M)>);

impl<M: Message> OutgoingMessages<M> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
}

/// User interface.
/// Methods should be called from inside of on_message handler.
impl<M: Message> OutgoingMessages<M> {
    pub fn broadcast(&mut self, message: M) {
        self.0.push((Destination::Broadcast, message));
    }

    pub fn send_to(&mut self, to: ProcessId, message: M) {
        self.0.push((Destination::To(to), message));
    }

    pub fn send_self(&mut self, message: M) {
        self.0.push((Destination::SendSelf, message));
    }
}
