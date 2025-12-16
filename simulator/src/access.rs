use crate::{Destination, Jiffies, ProcessId, communication::Message};

/// Injected into user's process's on_message handler.
/// This is common interface for process to communicate with simulation.
pub struct SimulationAccess<M: Message> {
    pub(crate) scheduled_events: Vec<(Destination, M)>,
    pub(crate) current_time: Jiffies,
}

impl<M: Message> SimulationAccess<M> {
    pub(crate) fn New(current_time: Jiffies) -> Self {
        Self {
            scheduled_events: Vec::new(),
            current_time,
        }
    }
}

/// User interface.
/// Methods should be called from inside of on_message handler.
impl<M: Message> SimulationAccess<M> {
    pub fn Broadcast(&mut self, message: M) {
        self.scheduled_events
            .push((Destination::Broadcast, message));
    }

    pub fn SendTo(&mut self, to: ProcessId, message: M) {
        self.scheduled_events.push((Destination::To(to), message));
    }

    pub fn SendSelf(&mut self, message: M) {
        self.scheduled_events.push((Destination::SendSelf, message));
    }

    pub fn CurrentTime(&self) -> Jiffies {
        self.current_time
    }
}
